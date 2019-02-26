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

use super::*;

use std::{collections::hash_map::Entry, error::Error,
          fmt::{self, Display, Formatter}, path::PathBuf, sync::{Arc, Weak}};

use capnp::{message::{Allocator, Builder}, OutputSegments, Word};
use fnv::{FnvBuildHasher, FnvHashMap};
use libc::{c_int, c_void, ptrdiff_t};
use lock_api::RwLockUpgradableReadGuard;
use parking_lot::{Condvar, Mutex, RwLock};
use rayon::prelude::*;

pub struct StaticCore<'a> {
    plugins: plugin_loader::PluginLoader,
    channels: Mutex<FnvHashMap<String, Weak<Channel>>>,
    nodes: FnvHashMap<String, Arc<node::Node<'a, 'a>>>,
    joined: Mutex<bool>,
    joined_cv: Condvar,
}

impl<'a> StaticCore<'a> {
    pub fn new(paths: Vec<PathBuf>) -> StaticCore<'a> {
        StaticCore{ plugins: plugin_loader::PluginLoader::new(paths),
                    channels: Mutex::new(FnvHashMap::with_hasher(FnvBuildHasher::default())),
                    nodes: FnvHashMap::with_hasher(FnvBuildHasher::default()),
                    joined: Mutex::new(false), joined_cv: Condvar::new() }
    }

    pub fn add_node(&mut self, name: String, tp: String) -> Result<(), NodeError> {
        // how is this safe?
        // Core functions only read or modify channels
        // in other words, we're orthogonally reading/modifying self
        // self.plugins/self.nodes are accessed here
        // aliased is used to modify self.channels
        let vptr = self.plugins.load(tp)
            .map_err(|e| NodeError::Load(e))?
            .vptr() as *const node::Vtbl;

        let aliased = unsafe { &mut *(self as *mut StaticCore) };
        let node = node::Node::new(aliased, unsafe { &*vptr })
            .map_err(|e| NodeError::Start(e))?;

        self.nodes.insert(name, Arc::new(node));

        Ok(())
    }

    pub fn run(&self) {
        let nodes: Vec<_> = self.nodes.values().cloned().collect();

        crossbeam::scope(move |s| {
            for node in nodes.into_iter() {
                s.spawn(move |_| node.run());
            }
        }).unwrap();

        let mut joined = self.joined.lock();
        *joined = true;

        self.joined_cv.notify_all();
    }

    pub fn stop(&self) {
        for node in self.nodes.values() {
            node.stop().unwrap();
        }

        let mut joined = self.joined.lock();

        while !*joined {
            self.joined_cv.wait(&mut joined);
        }
    }

    fn get_channel(&self, name: String, msg_type: u64) -> Result<Arc<Channel>, StaticCoreError> {
        let mut channels = self.channels.lock();

        let make_channel = |n| Arc::new(Channel::new(n, msg_type));

        match channels.entry(name) {
            Entry::Vacant(e) => {
                let channel = make_channel(e.key().clone());
                e.insert(Arc::downgrade(&channel));

                Ok(channel)
            },
            Entry::Occupied(mut e) => {
                match e.get_mut().upgrade() {
                    Some(c) => {
                        if c.msg_type() != msg_type {
                            return Err(StaticCoreError::ChannelTypeDiffers);
                        }

                        Ok(c)
                    }
                    None => {
                        let channel = make_channel(e.key().clone()); // all subscribers destroyed
                        e.insert(Arc::downgrade(&channel));

                        Ok(channel)
                    },
                }
            }
        }
    }
}

impl<'a> core::Core for StaticCore<'a> {
    type Error = StaticCoreError;
    type Publisher = Publisher;
    type Subscriber = Subscriber;

    fn get_type(&self) -> &'static str {
        "srm::static_core::StaticCore"
    }

    fn subscribe(&self, params: ffi::SubscribeParams)
    -> Result<Subscriber, StaticCoreError> {
        assert!(params.callback.is_some());

        let name = unsafe { ffi_to_str(params.topic) }.unwrap().to_string();
        let channel = self.get_channel(name, params.msg_type)?;

        Subscriber::new(channel, params.callback.unwrap(), params.arg)
            .ok_or(StaticCoreError::ChannelFull)
    }

    fn advertise(&self, params: ffi::AdvertiseParams) -> Result<Publisher, StaticCoreError> {
        let name = unsafe { ffi_to_str(params.topic) }.unwrap().to_string();
        let channel = self.get_channel(name, params.msg_type)?;

        Ok(Publisher{ channel })
    }

    srm_core_impl!(StaticCore);
}

pub struct Publisher {
    channel: Arc<Channel>,
}

impl core::Publisher for Publisher {
    type Allocator = alloc::CacheAlignedAllocator;
    type Error = StaticCoreError;

    fn get_channel_name(&self) -> &str {
        self.channel.name()
    }

    fn get_channel_type(&self) -> u64 {
        self.channel.msg_type()
    }

    fn publish(&mut self, builder: alloc::CacheAlignedBuilder) -> Result<(), StaticCoreError> {
        let weak_count = Arc::weak_count(&self.channel);

        if weak_count == 0 {
            return Err(StaticCoreError::ChannelDisconnected);
        }

        self.channel.publish(builder);

        Ok(())
    }

    srm_publisher_impl!(Publisher);
}

pub struct Subscriber {
    channel: Arc<Channel>,
    id: usize,
}

impl Subscriber {
    fn new(channel: Arc<Channel>, f: ffi::SubscribeCallback, arg: *mut c_void)
    -> Option<Subscriber> {
        let id = match channel.insert_callback(f, arg) {
            Some(i) => i,
            None => return None,
        };

        Some(Subscriber{ channel, id })
    }
}

impl core::Subscriber for Subscriber {
    type Error = StaticCoreError;

    fn get_channel_name(&self) -> &str {
        self.channel.name()
    }

    fn get_channel_type(&self) -> u64 {
        self.channel.msg_type()
    }

    srm_subscriber_impl!(Subscriber);
}

#[derive(Debug, Copy, Clone)]
pub enum StaticCoreError {
    OutOfMemory = 1,
    ChannelDisconnected,
    SubscriberDisconnected,
    ChannelFull,
    ChannelTypeDiffers,
}

impl core::Error for StaticCoreError {
    fn from_code(code: c_int) -> StaticCoreError {
        match code {
            1 => StaticCoreError::OutOfMemory,
            2 => StaticCoreError::ChannelDisconnected,
            3 => StaticCoreError::SubscriberDisconnected,
            4 => StaticCoreError::ChannelFull,
            5 => StaticCoreError::ChannelTypeDiffers,
            x => panic!("unknown code to construct StaticCoreError from: {}", x),
        }
    }

    fn as_code(&self) -> c_int {
        *self as c_int
    }

    fn what(&self) -> &'static str {
        match self {
            StaticCoreError::OutOfMemory => "out of memory",
            StaticCoreError::ChannelDisconnected => "channel disconnected",
            StaticCoreError::SubscriberDisconnected => "subscriber disconnected",
            StaticCoreError::ChannelFull => "channel has maximum subscribers",
            StaticCoreError::ChannelTypeDiffers
                => "channel exists, but has differing message type",
        }
    }
}

impl Error for StaticCoreError { }

impl Display for StaticCoreError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::core::Error;

        write!(f, "{} ({})", self.what(), self.as_code())
    }
}

struct Channel {
    name: String,
    msg_type: u64,
    max_num_callbacks: Option<usize>,
    callbacks: RwLock<(FnvHashMap<usize, Callback>, usize)>,
}

impl Channel {
    pub fn new(name: String, msg_type: u64) -> Channel {
        Channel{ name, msg_type, max_num_callbacks: None,
                 callbacks: RwLock::new((FnvHashMap::with_hasher(FnvBuildHasher::default()), 0)) }
    }

    pub fn with_max_callbacks(name: String, msg_type: u64, max_num_callbacks: usize) -> Channel {
        Channel{ name, msg_type, max_num_callbacks: Some(max_num_callbacks),
                 callbacks: RwLock::new((FnvHashMap::with_hasher(FnvBuildHasher::default()), 0)) }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn msg_type(&self) -> u64 {
        self.msg_type
    }

    pub fn insert_callback(&self, f: ffi::SubscribeCallback, arg: *mut c_void) -> Option<usize> {
        let mut callbacks = if let Some(max) = self.max_num_callbacks {
            let callbacks = self.callbacks.upgradable_read();

            if callbacks.0.len() == max {
                return None;
            }

            RwLockUpgradableReadGuard::upgrade(callbacks)
        } else {
            self.callbacks.write()
        };

        let id = callbacks.1;
        callbacks.1 += 1;

        callbacks.0.insert(id, Callback::new(f, arg));

        Some(id)
    }

    pub fn remove_callback(&self, id: usize) -> Option<()> {
        let mut callbacks = self.callbacks.write();

        callbacks.0.remove(&id).map(|_| { })
    }

    pub fn publish<A: Allocator + Send>(&self, builder: Builder<A>) -> usize {
        let segments = match builder.get_segments_for_output() {
            OutputSegments::SingleSegment(segment) => {
                vec![slice_to_msg_view(segment[0])]
            },
            OutputSegments::MultiSegment(segments) => {
                segments.iter().map(|s| slice_to_msg_view(*s)).collect()
            }
        };

        let msg = slice_to_msg(&segments, self.msg_type);

        let callbacks = self.callbacks.read();
        callbacks.0.par_iter().for_each(|(_, v)| {
            match unsafe { v.invoke(msg) } {
                0 => (),
                x => eprintln!("callback {:p} failed with errc {}", v.f, x),
            }
        });

        callbacks.0.len()
    }
}

#[derive(Debug)]
pub enum NodeError {
    Load(node_plugin::LoadError),
    Start(ErrorCode<'static>),
}

impl Error for NodeError { }

impl Display for NodeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            NodeError::Load(e) => write!(f, "load error: {}", e),
            NodeError::Start(e) => write!(f, "start error: {}", e),
        }
    }
}

fn slice_to_msg_view(slice: &[Word]) -> ffi::MsgSegmentView {
    assert!(slice.len() < ptrdiff_t::max_value() as usize);

    ffi::MsgSegmentView{ data: slice.as_ptr(), len: slice.len() as ffi::Index }
}

fn slice_to_msg(slice: &[ffi::MsgSegmentView], msg_type: u64) -> ffi::MsgView {
    ffi::MsgView{ segments: slice.as_ptr(), num_segments: slice.len() as ffi::Index, msg_type }
}

struct VeryUnsafePtr<T> {
    ptr: *const T,
}

unsafe impl<T> Send for VeryUnsafePtr<T> { }

unsafe impl<T> Sync for VeryUnsafePtr<T> { }

#[derive(Copy, Clone, Debug)]
struct Callback {
    f: ffi::SubscribeCallback,
    arg: *mut c_void,
}

impl Callback {
    fn new(f: ffi::SubscribeCallback, arg: *mut c_void) -> Callback {
        Callback{ f, arg }
    }

    unsafe fn invoke(&self, segments: ffi::MsgView) -> c_int {
        (self.f)(segments, self.arg)
    }
}

unsafe impl Send for Callback { }

unsafe impl Sync for Callback { }
