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
    pub impl_ptr: *const c_void,
    pub vptr: *const CoreVtbl,
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
    pub callback: Option<SubscribeCallback>,
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
#[allow(non_camel_case_types)]
pub enum ParamType {
    SRM_INTEGER,
    SRM_BOOLEAN,
    SRM_REAL,
    SRM_STRING,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CoreVtbl {
    pub get_type: Option<unsafe extern "C" fn(*const c_void) -> StrView>,

    pub subscribe:
        Option<unsafe extern "C" fn(*const c_void, SubscribeParams, *mut Subscriber) -> c_int>,
    pub advertise:
        Option<unsafe extern "C" fn(*const c_void, AdvertiseParams, *mut Publisher) -> c_int>,

    pub get_err_msg: Option<unsafe extern "C" fn(*const c_void, c_int) -> StrView>,

    pub log_error: Option<unsafe extern "C" fn(*const c_void, StrView) -> c_int>,
    pub log_warn: Option<unsafe extern "C" fn(*const c_void, StrView) -> c_int>,
    pub log_info: Option<unsafe extern "C" fn(*const c_void, StrView) -> c_int>,
    pub log_debug: Option<unsafe extern "C" fn(*const c_void, StrView) -> c_int>,
    pub log_trace: Option<unsafe extern "C" fn(*const c_void, StrView) -> c_int>,

    pub param_type: Option<unsafe extern "C" fn(*const c_void, StrView, *mut c_int) -> c_int>,

    pub param_seti: Option<unsafe extern "C" fn(*const c_void, StrView, isize) -> c_int>,
    pub param_geti: Option<unsafe extern "C" fn(*const c_void, StrView, *mut isize) -> c_int>,
    pub param_swapi:
        Option<unsafe extern "C" fn(*const c_void, StrView, isize, *mut isize) -> c_int>,

    pub param_setb: Option<unsafe extern "C" fn(*const c_void, StrView, c_int) -> c_int>,
    pub param_getb: Option<unsafe extern "C" fn(*const c_void, StrView, *mut c_int) -> c_int>,
    pub param_swapb:
        Option<unsafe extern "C" fn(*const c_void, StrView, c_int, *mut c_int) -> c_int>,

    pub param_setr: Option<unsafe extern "C" fn(*const c_void, StrView, f64) -> c_int>,
    pub param_getr: Option<unsafe extern "C" fn(*const c_void, StrView, *mut f64) -> c_int>,
    pub param_swapr: Option<unsafe extern "C" fn(*const c_void, StrView, f64, *mut f64) -> c_int>,

    pub param_sets: Option<unsafe extern "C" fn(*const c_void, StrView, StrView) -> c_int>,
    pub param_gets:
        Option<unsafe extern "C" fn(*const c_void, StrView, *mut util::String) -> c_int>,
    pub param_swaps:
        Option<unsafe extern "C" fn(*const c_void, StrView, StrView, *mut util::String) -> c_int>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PublisherVtbl {
    pub get_channel_name: Option<unsafe extern "C" fn(*const c_void) -> StrView>,
    pub get_channel_type: Option<unsafe extern "C" fn(*const c_void) -> MsgType>,
    pub publish: Option<unsafe extern "C" fn(*mut c_void, Option<PublishFn>, *mut c_void) -> c_int>,
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
