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

use std::{error::Error, fmt::{self, Display, Formatter}, sync::Arc};

use libc::c_int;

pub struct StaticCore {
    channels: Vec<Arc<Channel>>,
    publishers: Vec<Arc<Publisher>>,
    subscribers: Vec<Arc<Subscriber>>,
}

impl core::Core for StaticCore {
    type Error = StaticCoreError;
    type Publisher = Publisher;
    type Subscriber = Subscriber;

    fn get_type(&self) -> &'static str {
        "StaticCore"
    }

    fn subscribe(&mut self, params: ffi::SubscribeParams) -> Result<Subscriber, StaticCoreError> {

    }

    fn advertise(&mut self, params: ffi::AdvertiseParams) -> Result<Publisher, StaticCoreError> {

    }

    srm_core_impl!(StaticCore);
}

struct Channel {
    name: String,
    msg_type: u64,
}

impl Channel {
    pub fn get_name(&self) -> &str {
        self.name
    }

    pub fn get_type(&self) -> u64 {
        self.msg_type
    }
}

pub struct Publisher {
    channel: Arc<Channel>,
}

impl core::Publisher for Publisher {
    type Allocator = alloc::CacheAlignedAllocator;
    type Error = StaticCoreError;

    fn get_channel_name(&self) -> &str {
        self.channel.get_name()
    }

    fn get_channel_type(&self) -> u64 {
        self.channel.get_type()
    }
}

pub struct Subscriber {

}

impl core::Subscriber for Subscriber {
    type Error = StaticCoreError;
}

#[derive(Debug)]
pub enum StaticCoreError {
    Ok,
    OutOfMemory,
}

impl core::Error for StaticCoreError {
    fn from_code(code: c_int) -> StaticCoreError {
        assert!(code >= 0 && code < StaticCoreError::Size);

        code as isize as StaticCoreError
    }

    fn as_code(&self) -> c_int {
        self as c_int
    }

    fn what(&self) -> &'static str {
        match self {
            StaticCoreError::Ok => "ok",
            StaticCoreError::OutOfMemory => "out of memory",
        }
    }
}

impl Error for StaticCoreError { }

impl Display for StaticCoreError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.what(), self.as_code())
    }
}
