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

use std::marker::PhantomData;

use libc::{c_int, c_void};

#[repr(C)]
#[derive(Debug)]
pub struct Core<'a> {
    impl_ptr: *mut c_void,
    vptr: *const CoreVtbl<'a>,
    phantom: PhantomData<(&'a mut c_void, CoreVtbl<'a>)>,
}

impl<'a> Core<'a> {
    pub unsafe fn get_type(&'a self) -> &'a str {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let typestr = ((*self.vptr).get_type.unwrap())(self.impl_ptr);

        typestr.into_str().unwrap()
    }

    pub unsafe fn advertise(&'a mut self, params: AdvertiseParams) -> Result<'a, Publisher> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let mut publisher = Publisher{ impl_ptr: ptr::null_mut(), vptr: ptr::null(),
                                       phantom: PhantomData };
        let err = ((*self.vptr).advertise.unwrap())(self.impl_ptr, params, &mut publisher);

        match self.get_err_msg(err) {
            None => Ok(publisher),
            Some(e) => Err(ForeignError::new(err, e))
        }
    }

    pub unsafe fn subscribe(&'a mut self, params: SubscribeParams) -> Result<'a, Subscriber> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let mut subscriber = Subscriber{ impl_ptr: ptr::null_mut(), vptr: ptr::null(),
                                         phantom: PhantomData };
        let err = ((*self.vptr).subscribe.unwrap())(self.impl_ptr, params, &mut subscriber);

        match self.get_err_msg(err) {
            None => Ok(subscriber),
            Some(e) => Err(ForeignError::new(err, e))
        }
    }

    pub unsafe fn get_err_msg(&'a self, err: c_int) -> Option<&'a str> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        if err == 0 {
            None
        } else {
            Some(((*self.vptr).get_err_msg.unwrap())(self.impl_ptr, err).into_str().unwrap())
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Publisher<'a> {
    impl_ptr: *mut c_void,
    vptr: *const PublisherVtbl<'a>,
    phantom: PhantomData<(&'a mut c_void, PublisherVtbl<'a>)>,
}

impl<'a> Publisher<'a> {
    pub unsafe fn get_channel_name(&'a self) -> &'a str {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_channel_name.unwrap())(self.impl_ptr).into_str().unwrap()
    }

    pub unsafe fn get_channel_type(&'a self) -> MsgType {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_channel_type.unwrap())(self.impl_ptr)
    }

    pub unsafe fn publish(&'a mut self, publish_fn: PublishFn, arg: *mut c_void)
        -> Result<'a, ()> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let err = ((*self.vptr).publish.unwrap())(self.impl_ptr, publish_fn, arg);

        match self.get_err_msg(err) {
            None => Ok(()),
            Some(e) => Err(ForeignError::new(err, e))
        }
    }

    pub unsafe fn disconnect(&'a mut self) -> Result<'a, ()> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let err = ((*self.vptr).disconnect.unwrap())(self.impl_ptr);

        match self.get_err_msg(err) {
            None => Ok(()),
            Some(e) => Err(ForeignError::new(err, e))
        }
    }

    pub unsafe fn get_err_msg(&'a self, err: c_int) -> Option<&'a str> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        if err == 0 {
            None
        } else {
            Some(((*self.vptr).get_err_msg.unwrap())(self.impl_ptr, err).into_str().unwrap())
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Subscriber<'a> {
    impl_ptr: *mut c_void,
    vptr: *const SubscriberVtbl<'a>,
    phantom: PhantomData<(&'a mut c_void, SubscriberVtbl<'a>)>,
}

impl<'a> Subscriber<'a> {
    pub unsafe fn get_channel_name(&'a self) -> &'a str {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_channel_name.unwrap())(self.impl_ptr).into_str().unwrap()
    }

    pub unsafe fn get_channel_type(&'a self) -> MsgType {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_channel_type.unwrap())(self.impl_ptr)
    }

    pub unsafe fn disconnect(&'a mut self) -> Result<'a, ()> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        let err = ((*self.vptr).disconnect.unwrap())(self.impl_ptr);

        match self.get_err_msg(err) {
            None => Ok(()),
            Some(e) => Err(ForeignError::new(err, e))
        }
    }

    pub unsafe fn get_err_msg(&'a self, err: c_int) -> Option<&'a str> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        if err == 0 {
            None
        } else {
            Some(((*self.vptr).get_err_msg.unwrap())(self.impl_ptr, err).into_str().unwrap())
        }
    }
}

#[repr(C)]
pub struct SubscribeParams<'a> {
    msg_type: MsgType,
    topic: StrView<'a>,
    callback: SubscribeCallback,
    arg: *mut c_void,
}

#[repr(C)]
pub struct AdvertiseParams<'a> {
    msg_type: MsgType,
    topic: StrView<'a>,
}

#[repr(C)]
pub struct CoreVtbl<'a> {
    get_type: Option<extern "C" fn(*const c_void) -> StrView<'a>>,
    subscribe: Option<extern "C" fn(*mut c_void, SubscribeParams, *mut Subscriber) -> c_int>,
    advertise: Option<extern "C" fn(*mut c_void, AdvertiseParams, *mut Publisher) -> c_int>,
    get_err_msg: Option<extern "C" fn(*mut c_void, c_int) -> StrView<'a>>,
}

impl<'a> CoreVtbl<'a> {
    pub fn is_non_null(self) -> bool {
        self.get_type.is_some() && self.subscribe.is_some()
            && self.advertise.is_some() && self.get_err_msg.is_some()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PublisherVtbl<'a> {
    get_channel_name: Option<extern "C" fn(*const c_void) -> StrView<'a>>,
    get_channel_type: Option<extern "C" fn(*const c_void) -> MsgType>,
    publish: Option<extern "C" fn(*mut c_void, PublishFn, *mut c_void) -> c_int>,
    disconnect: Option<extern "C" fn(*mut c_void) -> c_int>,
    get_err_msg: Option<extern "C" fn(*const c_void, c_int) -> StrView<'a>>,
}

impl<'a> PublisherVtbl<'a> {
    pub fn is_non_null(self) -> bool {
        self.get_channel_name.is_some() && self.get_channel_type.is_some()
            && self.publish.is_some() && self.disconnect.is_some() && self.get_err_msg.is_some()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SubscriberVtbl<'a> {
    get_channel_name: Option<extern "C" fn(*const c_void) -> StrView<'a>>,
    get_channel_type: Option<extern "C" fn(*const c_void) -> u64>,
    disconnect: Option<extern "C" fn(*mut c_void) -> c_int>,
    get_err_msg: Option<extern "C" fn(*const c_void, c_int) -> StrView<'a>>,
}

impl<'a> SubscriberVtbl<'a> {
    pub fn is_non_null(self) -> bool {
        self.get_channel_name.is_some() && self.get_channel_type.is_some()
            && self.disconnect.is_some() && self.get_err_msg.is_some()
    }
}
