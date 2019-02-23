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

use libc::{c_int, c_void};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Core {
    impl_ptr: *mut c_void,
    vptr: *const CoreVtbl,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Publisher {
    pub impl_ptr: *mut c_void,
    pub vptr: *const PublisherVtbl,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Subscriber {
    pub impl_ptr: *mut c_void,
    pub vptr: *const SubscriberVtbl,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SubscribeParams {
    pub msg_type: MsgType,
    pub topic: StrView,
    pub callback: SubscribeCallback,
    pub arg: *mut c_void,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AdvertiseParams {
    pub msg_type: MsgType,
    pub topic: StrView,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CoreVtbl {
    pub get_type: Option<unsafe extern "C" fn(*const c_void) -> StrView>,
    pub subscribe: Option<unsafe extern "C" fn(*mut c_void, SubscribeParams, *mut Subscriber) -> c_int>,
    pub advertise: Option<unsafe extern "C" fn(*mut c_void, AdvertiseParams, *mut Publisher) -> c_int>,
    pub get_err_msg: Option<unsafe extern "C" fn(*const c_void, c_int) -> StrView>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PublisherVtbl {
    pub get_channel_name: Option<unsafe extern "C" fn(*const c_void) -> StrView>,
    pub get_channel_type: Option<unsafe extern "C" fn(*const c_void) -> MsgType>,
    pub publish: Option<unsafe extern "C" fn(*mut c_void, PublishFn, *mut c_void) -> c_int>,
    pub disconnect: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub get_err_msg: Option<unsafe extern "C" fn(*const c_void, c_int) -> StrView>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SubscriberVtbl {
    pub get_channel_name: Option<unsafe extern "C" fn(*const c_void) -> StrView>,
    pub get_channel_type: Option<unsafe extern "C" fn(*const c_void) -> u64>,
    pub disconnect: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub get_err_msg: Option<unsafe extern "C" fn(*const c_void, c_int) -> StrView>,
}
