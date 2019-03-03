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

use super::{
    alloc::CacheAlignedAllocator,
    core::{self, MessageBuilder},
    ffi,
    node::Node,
    plugin_loader::PluginLoader,
    util::ffi_to_str,
    *,
};

use std::{
    cell::UnsafeCell,
    error::Error,
    fmt::{self, Display, Formatter},
    path::PathBuf,
    sync::{Arc, Weak},
};

use hashbrown::{hash_map::Entry, HashMap};
use libc::{c_int, c_void};
use lock_api::RwLockUpgradableReadGuard;
use log::{debug, error, info, trace, warn};
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;

pub struct Core {
    plugin_loader: Mutex<PluginLoader>,
    channels: Mutex<HashMap<String, Weak<Channel>>>,
    nodes: RwLock<HashMap<String, Arc<CoreInterface>>>,
}

pub fn add_node(core: Arc<Core>, name: String, tp: String) -> Result<(), NodeError> {
    let plugin = {
        let mut plugin_loader = core.plugin_loader.lock();
        plugin_loader.load(tp).map_err(|e| NodeError::Load(e))?
    };

    let interface = Arc::new(CoreInterface {
        core: Arc::downgrade(&core),
        node: UnsafeCell::new(Arc::new(Node::new(plugin, name.clone()))),
    });

    Arc::get_mut(unsafe { interface.node_mut() })
        .unwrap()
        .start(interface.clone())
        .map_err(|e| NodeError::Start(e))?;

    let was_present = {
        let mut nodes = core.nodes.write();
        nodes.insert(name, interface).is_some()
    };
    assert!(!was_present);

    Ok(())
}

impl Core {
    pub fn new(paths: Vec<PathBuf>) -> Core {
        Core {
            plugin_loader: Mutex::new(PluginLoader::new(paths)),
            channels: Mutex::new(HashMap::new()),
            nodes: RwLock::new(HashMap::new()),
        }
    }

    pub fn run(&self) {
        let nodes: Vec<Arc<Node>> = {
            let nodes = self.nodes.read();
            nodes.iter().map(|(_, c)| c.node().clone()).collect()
        };

        crossbeam::scope(move |s| {
            for node in nodes.into_iter() {
                s.spawn(move |_| node.run().unwrap());
            }
        })
        .unwrap();
    }

    pub fn stop(&self) {
        let interfaces = self.nodes.read();

        for i in interfaces.values() {
            i.node().stop().unwrap();
        }
    }

    pub fn subscribe(&self, params: ffi::SubscribeParams) -> Result<Subscriber, StaticCoreError> {
        assert!(params.callback.is_some());

        let name = unsafe { ffi_to_str(params.topic) }.unwrap().to_string();
        let channel = self.get_channel(name, params.msg_type)?;

        Subscriber::new(channel, params.callback.unwrap(), params.arg)
            .ok_or(StaticCoreError::ChannelFull)
    }

    pub fn advertise(&self, params: ffi::AdvertiseParams) -> Result<Publisher, StaticCoreError> {
        let name = unsafe { ffi_to_str(params.topic) }.unwrap().to_string();
        let channel = self.get_channel(name, params.msg_type)?;

        Ok(Publisher { channel })
    }

    fn get_channel(&self, name: String, msg_type: u64) -> Result<Arc<Channel>, StaticCoreError> {
        let mut channels = self.channels.lock();

        let make_channel = |n| Arc::new(Channel::new(n, msg_type));

        match channels.entry(name) {
            Entry::Vacant(e) => {
                let channel = make_channel(e.key().clone());
                e.insert(Arc::downgrade(&channel));

                Ok(channel)
            }
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
                    }
                }
            }
        }
    }
}

struct CoreInterface {
    core: Weak<Core>,
    node: UnsafeCell<Arc<Node>>,
}

impl CoreInterface {
    fn node(&self) -> &Arc<Node> {
        unsafe { &*self.node.get() }
    }

    unsafe fn node_mut(&self) -> &mut Arc<Node> {
        &mut *self.node.get()
    }

    fn name(&self) -> &str {
        self.node().name()
    }
}

impl core::Core for CoreInterface {
    type Error = StaticCoreError;
    type Publisher = Publisher;
    type Subscriber = Subscriber;

    fn get_type(&self) -> &'static str {
        "srm::static_core::CoreInterface"
    }

    fn subscribe(&self, params: ffi::SubscribeParams) -> Result<Subscriber, StaticCoreError> {
        assert!(self.core.upgrade().is_some());

        self.core.upgrade().unwrap().subscribe(params)
    }

    fn advertise(&self, params: ffi::AdvertiseParams) -> Result<Publisher, StaticCoreError> {
        self.core.upgrade().unwrap().advertise(params)
    }

    fn log_error(&self, msg: &str) -> Result<(), StaticCoreError> {
        error!(target: self.name(), "{}", msg);

        Ok(())
    }

    fn log_warn(&self, msg: &str) -> Result<(), StaticCoreError> {
        warn!(target: self.name(), "{}", msg);

        Ok(())
    }

    fn log_info(&self, msg: &str) -> Result<(), StaticCoreError> {
        info!(target: self.name(), "{}", msg);

        Ok(())
    }

    fn log_debug(&self, msg: &str) -> Result<(), StaticCoreError> {
        debug!(target: self.name(), "{}", msg);

        Ok(())
    }

    fn log_trace(&self, msg: &str) -> Result<(), StaticCoreError> {
        trace!(target: self.name(), "{}", msg);

        Ok(())
    }
}

impl core::CoreBase for CoreInterface {
    srm_core_base_impl!(CoreInterface);
}

unsafe impl Send for CoreInterface {}

unsafe impl Sync for CoreInterface {}

pub struct Publisher {
    channel: Arc<Channel>,
}

impl core::Publisher for Publisher {
    type Builder = CacheAlignedAllocator;
    type Error = StaticCoreError;

    fn get_channel_name(&self) -> &str {
        self.channel.name()
    }

    fn get_channel_type(&self) -> u64 {
        self.channel.msg_type()
    }

    fn publish(&mut self, allocator: CacheAlignedAllocator) -> Result<(), StaticCoreError> {
        let weak_count = Arc::weak_count(&self.channel);

        if weak_count == 0 {
            return Err(StaticCoreError::ChannelDisconnected);
        }

        self.channel.publish(allocator);

        Ok(())
    }

    fn get_allocator(&self) -> CacheAlignedAllocator {
        CacheAlignedAllocator::new()
    }

    srm_publisher_impl!(Publisher);
}

pub struct Subscriber {
    channel: Arc<Channel>,
    id: usize,
}

impl Subscriber {
    fn new(
        channel: Arc<Channel>,
        f: ffi::SubscribeCallback,
        arg: *mut c_void,
    ) -> Option<Subscriber> {
        let id = match channel.insert_callback(f, arg) {
            Some(i) => i,
            None => return None,
        };

        Some(Subscriber { channel, id })
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

impl Drop for Subscriber {
    fn drop(&mut self) {
        self.channel.remove_callback(self.id);
    }
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
            StaticCoreError::ChannelTypeDiffers => "channel exists, but has differing message type",
        }
    }
}

impl Error for StaticCoreError {}

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
    callbacks: Arc<RwLock<(Vec<(usize, Callback)>, usize)>>,
}

impl Channel {
    pub fn new(name: String, msg_type: u64) -> Channel {
        Channel {
            name,
            msg_type,
            max_num_callbacks: None,
            callbacks: Arc::new(RwLock::new((Vec::with_capacity(8), 0))),
        }
    }

    pub fn with_max_callbacks(name: String, msg_type: u64, max_num_callbacks: usize) -> Channel {
        Channel {
            name,
            msg_type,
            max_num_callbacks: Some(max_num_callbacks),
            callbacks: Arc::new(RwLock::new((Vec::with_capacity(max_num_callbacks), 0))),
        }
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

        callbacks.0.push((id, Callback::new(f, arg)));

        Some(id)
    }

    pub fn remove_callback(&self, id: usize) -> Option<()> {
        let callbacks = self.callbacks.upgradable_read();

        let index = match callbacks.0.iter().position(|(i, _)| *i == id) {
            None => return None,
            Some(i) => i,
        };

        let mut callbacks = RwLockUpgradableReadGuard::upgrade(callbacks);

        callbacks.0.remove(index);

        Some(())
    }

    pub fn publish(&self, allocator: alloc::CacheAlignedAllocator) {
        let callbacks = self.callbacks.read();
        Channel::do_publish(allocator, &callbacks.0, self.msg_type);
    }

    pub fn publish_nonblocking(&self, allocator: alloc::CacheAlignedAllocator) {
        let callbacks = self.callbacks.clone();
        let msg_type = self.msg_type;

        rayon::spawn(move || {
            let guard = callbacks.read();
            Channel::do_publish(allocator, &guard.0, msg_type)
        });
    }

    fn do_publish(
        allocator: alloc::CacheAlignedAllocator,
        callbacks: &Vec<(usize, Callback)>,
        msg_type: u64,
    ) {
        let segments = unsafe { allocator.as_view() };
        let msg = slice_to_msg(&segments, msg_type);

        callbacks
            .par_iter()
            .for_each(|(_, c)| match unsafe { c.invoke(msg) } {
                0 => (),
                x => eprintln!("callback {:p} failed with errc {}", c.f, x),
            });
    }
}

#[derive(Debug)]
pub enum NodeError {
    Load(node_plugin::LoadError),
    Start(ErrorCode),
}

impl Error for NodeError {}

impl Display for NodeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            NodeError::Load(e) => write!(f, "load error: {}", e),
            NodeError::Start(e) => write!(f, "start error: {}", e),
        }
    }
}

fn slice_to_msg(slice: &[ffi::MsgSegmentView], msg_type: u64) -> ffi::MsgView {
    ffi::MsgView {
        segments: slice.as_ptr(),
        num_segments: slice.len() as ffi::Index,
        msg_type,
    }
}

#[derive(Copy, Clone, Debug)]
struct Callback {
    f: ffi::SubscribeCallback,
    arg: *mut c_void,
}

impl Callback {
    fn new(f: ffi::SubscribeCallback, arg: *mut c_void) -> Callback {
        Callback { f, arg }
    }

    unsafe fn invoke(&self, segments: ffi::MsgView) -> c_int {
        (self.f)(segments, self.arg)
    }
}

unsafe impl Send for Callback {}

unsafe impl Sync for Callback {}
