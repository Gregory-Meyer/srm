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

use std::{error::Error, fmt::{self, Display, Formatter}, sync::{Arc, Weak}};

use capnp::{message::{Allocator, Builder}, OutputSegments, Word};
use fnv::{FnvBuildHasher, FnvHashMap, FnvHashSet};
use libc::{c_int, c_void, ptrdiff_t};
use lock_api::RwLockUpgradableReadGuard;
use parking_lot::RwLock;
use rayon::prelude::*;

pub struct StaticCore {
    channels: FnvHashSet<Weak<Channel>>,
}

impl core::Core for StaticCore {
    type Error = StaticCoreError;
    type Publisher = Publisher;
    type Subscriber = Subscriber;

    fn get_type(&self) -> &str {
        unimplemented!()
    }

    fn subscribe(&mut self, params: ffi::SubscribeParams)
    -> Result<Subscriber, StaticCoreError> {
        unimplemented!()
    }

    fn advertise(&mut self, params: ffi::AdvertiseParams) -> Result<Self::Publisher, Self::Error> {
        unimplemented!()
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
        unimplemented!()
    }

    fn get_channel_type(&self) -> u64 {
        unimplemented!()
    }

    fn publish(&mut self, builder: alloc::CacheAlignedBuilder) -> Result<(), StaticCoreError> {
        let weak_count = Arc::weak_count(&self.channel);

        if weak_count == 0 {
            return Err(StaticCoreError::ChannelDisconnected);
        }

        self.channel.publish(builder);

        Ok(())
    }

    fn into_ffi(self) -> ffi::Publisher {
        unimplemented!()
    }
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

    fn into_ffi(self) -> ffi::Subscriber {
        unimplemented!()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum StaticCoreError {
    OutOfMemory = 1,
    ChannelDisconnected,
    SubscriberDisconnected,
}

impl core::Error for StaticCoreError {
    fn from_code(code: c_int) -> StaticCoreError {
        match code {
            1 => StaticCoreError::OutOfMemory,
            2 => StaticCoreError::ChannelDisconnected,
            3 => StaticCoreError::SubscriberDisconnected,
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

fn slice_to_msg_view(slice: &[Word]) -> ffi::MsgSegmentView {
    assert!(slice.len() < ptrdiff_t::max_value() as usize);

    ffi::MsgSegmentView{ data: slice.as_ptr(), len: slice.len() as ffi::Index }
}

fn slice_to_msg(slice: &[ffi::MsgSegmentView], msg_type: u64) -> ffi::MsgView {
    ffi::MsgView{ segments: slice.as_ptr(), num_segments: slice.len() as ffi::Index, msg_type }
}

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
