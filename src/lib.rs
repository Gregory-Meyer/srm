#![allow(dead_code)]

extern crate capnp;
extern crate fnv;
extern crate libc;
extern crate libloading;
extern crate parking_lot;
extern crate rayon;

use std::{collections::HashMap, error::Error, fmt::{self, Display, Formatter},
          str, sync::Arc, path::{Path, PathBuf}};

use capnp::{message::ReaderSegments, serialize::OwnedSegments};
use fnv::FnvBuildHasher;
use parking_lot::RwLock;
use rayon::prelude::*;
use libloading::{Library, Symbol};

mod ffi {
    use libc::{c_char, c_int, c_void, ptrdiff_t};

    #[repr(C)]
    struct Core {
        impl_ptr: *mut c_void,
        vptr: *const CoreVtbl,
    }

    #[repr(C)]
    struct Publisher {
        impl_ptr: *mut c_void,
        vptr: *const PublisherVtbl,
    }

    #[repr(C)]
    struct Subscriber {
        impl_ptr: *mut c_void,
        vptr: *const SubscriberVtbl,
    }

    #[repr(C)]
    struct SubscribeParams {
        msg_type: u64,
        topic: StrView,
        callback: extern "C" fn(Core, MsgView, *mut c_void) -> c_int,
        arg: *mut c_void,
    }

    #[repr(C)]
    struct StrView {
        data: *const c_char,
        len: ptrdiff_t,
    }

    #[repr(C)]
    struct CoreVtbl {
        get_type: extern "C" fn(*const c_void) -> StrView,
        subscribe: extern "C" fn(*mut c_void, SubscribeParams, *mut Subscriber),
    }

    #[repr(C)]
    struct PublisherVtbl {
        get_channel_name: extern "C" fn(*const c_void) -> StrView,
        get_channel_type: extern "C" fn(*const c_void) -> u64,
    }

    #[repr(C)]
    struct SubscriberVtbl {
        get_channel_name: extern "C" fn(*const c_void) -> StrView,
        get_channel_type: extern "C" fn(*const c_void) -> u64,
    }

    #[repr(C)]
    struct MsgView {
        segments: *const SegmentView,
        num_segments: ptrdiff_t,
        msg_type: u64,
    }

    #[repr(C)]
    struct SegmentView {
        data: *const u64,
        len: ptrdiff_t,
    }
}

pub struct Core {
    loader: PluginLoader,
}

struct PluginLoader {
    paths: Vec<PathBuf>,
    plugins: HashMap<String, Arc<Library>, FnvBuildHasher>,
}

pub struct Channel {
    name: String,
    msg_type: u64,
    callbacks: HashMap<usize, Box<dyn Fn(&ReaderSegments)
        -> Result<(), (i32, &'static [u8])> + Sync + Send>, FnvBuildHasher>,
    id_counter: usize,
    max_subscribers: Option<usize>,
}

#[derive(Debug)]
struct MaxSubscribersReachedError { }

impl Error for MaxSubscribersReachedError { }

impl Display for MaxSubscribersReachedError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "channel max subscribers reached")
    }
}

impl Channel {
    fn new(name: String, msg_type: u64) -> Channel {
        Channel{ name, msg_type, callbacks: HashMap::with_hasher(FnvBuildHasher::default()),
                 id_counter: 0, max_subscribers: None }
    }

    fn with_bound(name: String, msg_type: u64, max_subscribers: usize) -> Channel {
        Channel{ name, msg_type, callbacks: HashMap::with_hasher(FnvBuildHasher::default()),
                 id_counter: 0, max_subscribers: Some(max_subscribers) }
    }

    fn add_callback<F>(&mut self, callback: F) -> Result<usize, MaxSubscribersReachedError>
        where F: Fn(&ReaderSegments) -> Result<(), (i32, &'static [u8])> + Sync + Send + 'static {
        if let Some(m) = self.max_subscribers {
            if self.callbacks.len() == m {
                return Err(MaxSubscribersReachedError{ });
            }
        }

        let id = self.id_counter;
        self.id_counter += 1;

        self.callbacks.insert(id, Box::new(callback));

        Ok(id)
    }
}

pub struct Subscriber {
    channel: Arc<RwLock<Channel>>,
    id: usize,
}

impl Subscriber {
    fn new<F>(channel: Arc<RwLock<Channel>>, callback: F)
        -> Result<Subscriber, MaxSubscribersReachedError>
        where F: Fn(&ReaderSegments) -> Result<(), (i32, &'static [u8])> + Sync + Send + 'static {
        let id = {
            let mut guard = channel.write();
            guard.add_callback(callback)?
        };

        Ok(Subscriber{ channel, id })
    }

    fn msg_type(&self) -> u64 {
        let channel = self.channel.read();

        channel.msg_type
    }
}

impl<'a> Subscriber {
    fn name(&'a self) -> &'a str {
        let channel = self.channel.read();

        let str_ptr: *const _ = &channel.name as &str;

        unsafe { &(*str_ptr) }
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        let mut channel = self.channel.write();
        channel.callbacks.remove(&self.id);
    }
}

pub struct Publisher {
    channel: Arc<RwLock<Channel>>,
}

impl Publisher {
    fn new(channel: Arc<RwLock<Channel>>) -> Publisher {
        Publisher{ channel }
    }

    fn publish(&self, segments: OwnedSegments) {
        let channel = self.channel.clone();

        rayon::spawn(move || {
            let channel = channel.read();

            channel.callbacks.par_iter().map(|(_, v)| v).for_each(|callback| {
                if let Err((c, m)) = callback(&segments) {
                    let msg_str = unsafe { str::from_utf8_unchecked(&m[..m.len()]) };
                    eprintln!("error handling callback on channel {}: {} ({})",
                              channel.name, c, msg_str);
                }
            });
        });
    }

    fn msg_type(&self) -> u64 {
        let channel = self.channel.read();

        channel.msg_type
    }
}

impl<'a> Publisher {
    fn name(&'a self) -> &'a str {
        let channel = self.channel.read();

        let str_ptr: *const _ = &channel.name as &str;

        unsafe { &(*str_ptr) }
    }
}