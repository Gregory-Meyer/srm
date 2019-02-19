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

use std::ptr;

use libc::{c_int, c_void};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Core {
    impl_ptr: *mut c_void,
    vptr: *const CoreVtbl,
}

impl Core {
    pub unsafe fn get_type(self) -> StrView {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_type.unwrap())(self.impl_ptr)
    }

    pub unsafe fn subscribe(self, params: SubscribeParams)
        -> Result<Subscriber, (c_int, StrView)> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let mut subscriber = Subscriber::default();
        let err = ((*self.vptr).subscribe.unwrap())(self.impl_ptr, params, &mut subscriber);

        if err != 0 {
            Err((err, self.get_err_msg(err)))
        } else {
            Ok(subscriber)
        }
    }

    pub unsafe fn advertise(self, params: AdvertiseParams) -> Result<Publisher, (c_int, StrView)> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let mut publisher = Publisher::default();
        let err = ((*self.vptr).advertise.unwrap())(self.impl_ptr, params, &mut publisher);

        if err != 0 {
            Err((err, self.get_err_msg(err)))
        } else {
            Ok(publisher)
        }
    }

    pub unsafe fn get_err_msg(self, err: c_int) -> StrView {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_err_msg.unwrap())(self.impl_ptr, err)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Publisher {
    impl_ptr: *mut c_void,
    vptr: *const PublisherVtbl,
}

impl Default for Publisher {
    fn default() -> Publisher {
        Publisher{ impl_ptr: ptr::null_mut(), vptr: ptr::null() }
    }
}

impl Publisher {
    pub unsafe fn get_channel_name(self) -> StrView {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_channel_name.unwrap())(self.impl_ptr)
    }

    pub unsafe fn get_channel_type(self) -> MsgType {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_channel_type.unwrap())(self.impl_ptr)
    }

    pub unsafe fn publish(self, publish_fn: PublishFn, arg: *mut c_void)
        -> Result<(), (c_int, StrView)> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let err = ((*self.vptr).publish.unwrap())(self.impl_ptr, publish_fn, arg);

        if err != 0 {
            Err((err, self.get_err_msg(err)))
        } else {
            Ok(())
        }
    }

    pub unsafe fn disconnect(self) -> Result<(), (c_int, StrView)> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let err = ((*self.vptr).disconnect.unwrap())(self.impl_ptr);

        if err != 0 {
            Err((err, self.get_err_msg(err)))
        } else {
            Ok(())
        }
    }

    pub unsafe fn get_err_msg(self, err: c_int) -> StrView {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_err_msg.unwrap())(self.impl_ptr, err)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Subscriber {
    impl_ptr: *mut c_void,
    vptr: *const SubscriberVtbl,
}

impl Default for Subscriber {
    fn default() -> Subscriber {
        Subscriber{ impl_ptr: ptr::null_mut(), vptr: ptr::null() }
    }
}

impl Subscriber {
    pub unsafe fn get_channel_name(self) -> StrView {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_channel_name.unwrap())(self.impl_ptr)
    }

    pub unsafe fn get_channel_type(self) -> MsgType {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_channel_type.unwrap())(self.impl_ptr)
    }

    pub unsafe fn disconnect(self) -> Result<(), (c_int, StrView)> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let err = ((*self.vptr).disconnect.unwrap())(self.impl_ptr);

        if err != 0 {
            Err((err, self.get_err_msg(err)))
        } else {
            Ok(())
        }
    }

    pub unsafe fn get_err_msg(self, err: c_int) -> StrView {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_err_msg.unwrap())(self.impl_ptr, err)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SubscribeParams {
    msg_type: MsgType,
    topic: StrView,
    callback: SubscribeCallback,
    arg: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AdvertiseParams {
    msg_type: MsgType,
    topic: StrView,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CoreVtbl {
    get_type: Option<extern "C" fn(*const c_void) -> StrView>,
    subscribe: Option<extern "C" fn(*mut c_void, SubscribeParams, *mut Subscriber) -> c_int>,
    advertise: Option<extern "C" fn(*mut c_void, AdvertiseParams, *mut Publisher) -> c_int>,
    get_err_msg: Option<extern "C" fn(*mut c_void, c_int) -> StrView>,
}

impl CoreVtbl {
    pub fn is_non_null(self) -> bool {
        self.get_type.is_some() && self.subscribe.is_some()
            && self.advertise.is_some() && self.get_err_msg.is_some()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PublisherVtbl {
    get_channel_name: Option<extern "C" fn(*const c_void) -> StrView>,
    get_channel_type: Option<extern "C" fn(*const c_void) -> MsgType>,
    publish: Option<extern "C" fn(*mut c_void, PublishFn, *mut c_void) -> c_int>,
    disconnect: Option<extern "C" fn(*mut c_void) -> c_int>,
    get_err_msg: Option<extern "C" fn(*const c_void, c_int) -> StrView>,
}

impl PublisherVtbl {
    pub fn is_non_null(self) -> bool {
        self.get_channel_name.is_some() && self.get_channel_type.is_some()
            && self.publish.is_some() && self.disconnect.is_some() && self.get_err_msg.is_some()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SubscriberVtbl {
    get_channel_name: Option<extern "C" fn(*const c_void) -> StrView>,
    get_channel_type: Option<extern "C" fn(*const c_void) -> u64>,
    disconnect: Option<extern "C" fn(*mut c_void) -> c_int>,
    get_err_msg: Option<extern "C" fn(*const c_void, c_int) -> StrView>,
}

impl SubscriberVtbl {
    pub fn is_non_null(self) -> bool {
        self.get_channel_name.is_some() && self.get_channel_type.is_some()
            && self.disconnect.is_some() && self.get_err_msg.is_some()
    }
}
