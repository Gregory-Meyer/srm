// Copyright 2019 Gregory Meyer
//
// Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation
// files (the "Software"), to deal in the Software without
// restriction, including without limitation the rights to use, copy,
// modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
// BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
// ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![allow(dead_code)]

extern crate capnp;
extern crate fnv;
extern crate libc;
extern crate libloading;
extern crate lock_api;
extern crate parking_lot;
extern crate rayon;

use std::{collections::HashMap, error::Error, fmt::{self, Display, Formatter},
          str, sync::Arc, path::{Path, PathBuf}};

use capnp::{message::ReaderSegments, serialize::OwnedSegments};
use fnv::FnvBuildHasher;
use parking_lot::RwLock;
use rayon::prelude::*;
use libloading::{Library, Symbol};
use lock_api::RwLockUpgradableReadGuard;

pub mod ffi;

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
    max_subscribers: Option<usize>,
    state: RwLock<ChannelMutableState>,
}

struct ChannelMutableState {
    callbacks: HashMap<usize, Box<dyn Fn(&ReaderSegments)
        -> Result<(), (i32, &'static [u8])> + Sync + Send>, FnvBuildHasher>,
    id_counter: usize,
}

impl ChannelMutableState {
    fn new() -> ChannelMutableState {
        ChannelMutableState{ callbacks: HashMap::with_hasher(FnvBuildHasher::default()),
                             id_counter: 0 }
    }
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
        Channel{ name, msg_type, max_subscribers: None,
                 state: RwLock::new(ChannelMutableState::new()) }
    }

    fn with_bound(name: String, msg_type: u64, max_subscribers: usize) -> Channel {
        Channel{ name, msg_type, max_subscribers: Some(max_subscribers),
                 state: RwLock::new(ChannelMutableState::new()) }
    }

    fn add_callback<F>(&self, callback: F) -> Result<usize, MaxSubscribersReachedError>
        where F: Fn(&ReaderSegments) -> Result<(), (i32, &'static [u8])> + Sync + Send + 'static {
        let state = self.state.upgradable_read();

        if let Some(m) = self.max_subscribers {
            if state.callbacks.len() == m {
                return Err(MaxSubscribersReachedError{ });
            }
        }

        let mut state = RwLockUpgradableReadGuard::upgrade(state);

        let id = state.id_counter;
        state.id_counter += 1;

        state.callbacks.insert(id, Box::new(callback));

        Ok(id)
    }

    fn remove_callback(&self, id: usize) -> bool {
        let mut state = self.state.write();

        state.callbacks.remove(&id).is_some()
    }

    fn send_message(&self, segments: OwnedSegments) {
        let state = self.state.read();

        state.callbacks.par_iter().map(|(_, v)| v).for_each(|callback| {
            if let Err((c, m)) = callback(&segments) {
                let msg_str = unsafe { str::from_utf8_unchecked(&m[..m.len()]) };
                eprintln!("error handling callback on channel {}: {} ({})",
                          self.name, c, msg_str);
            }
        });
    }
}

pub struct Subscriber {
    channel: Arc<Channel>,
    id: usize,
}

impl Subscriber {
    fn new<F>(channel: Arc<Channel>, callback: F)
        -> Result<Subscriber, MaxSubscribersReachedError>
        where F: Fn(&ReaderSegments) -> Result<(), (i32, &'static [u8])> + Sync + Send + 'static {
        let id = channel.add_callback(callback)?;

        Ok(Subscriber{ channel, id })
    }

    fn msg_type(&self) -> u64 {
        self.channel.msg_type
    }
}

impl<'a> Subscriber {
    fn name(&'a self) -> &'a str {
        let str_ptr: *const _ = &self.channel.name as &str;

        unsafe { &(*str_ptr) }
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        self.channel.remove_callback(self.id);
    }
}

pub struct Publisher {
    channel: Arc<Channel>,
}

impl Publisher {
    fn new(channel: Arc<Channel>) -> Publisher {
        Publisher{ channel }
    }

    fn publish(&self, segments: OwnedSegments) {
        let channel = self.channel.clone();

        rayon::spawn(move || {
            channel.send_message(segments);
        });
    }

    fn msg_type(&self) -> u64 {
        self.channel.msg_type
    }
}

impl<'a> Publisher {
    fn name(&'a self) -> &'a str {
        let str_ptr: *const _ = &self.channel.name as &str;

        unsafe { &(*str_ptr) }
    }
}
