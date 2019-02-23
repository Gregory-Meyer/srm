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

use std::error;

use capnp::message::{Allocator, Builder};
use libc::{c_char, c_int, c_void, ptrdiff_t};

pub trait Error: error::Error {
    fn from_code(code: c_int) -> Self;

    fn as_code(&self) -> c_int;

    fn what(&self) -> &'static str;
}

pub trait Publisher: Drop {
    type Allocator: Allocator;
    type Error: Error;

    fn get_channel_name(&self) -> &str;

    fn get_channel_type(&self) -> u64;

    fn publish(&mut self, builder: Builder<Self::Allocator>) -> Result<(), Self::Error>;

    fn disconnect(&mut self) -> Result<(), Self::Error>;

    fn into_ffi(self) -> ffi::Publisher;
}

pub trait Subscriber: Drop {
    type Error: Error;

    fn get_channel_name(&self) -> &str;

    fn get_channel_type(&self) -> u64;

    fn disconnect(&mut self) -> Result<(), Self::Error>;

    fn into_ffi(self) -> ffi::Subscriber;
}

pub trait Core {
    type Error: Error;
    type Publisher: Publisher;
    type Subscriber: Subscriber;

    fn get_type(&self) -> &str;

    fn subscribe(&mut self, params: ffi::SubscribeParams) -> Result<Self::Subscriber, Self::Error>;

    fn advertise(&mut self, params: ffi::AdvertiseParams) -> Result<Self::Publisher, Self::Error>;

    fn as_ffi(&mut self) -> ffi::Core;
}

#[macro_export]
macro_rules! srm_core_impl {
    ($x:ty) => (
        fn as_ffi(&mut self) -> ffi::Core {
            const VTBL: ffi::CoreVtbl = ffi::CoreVtbl{
                get_type: Some(get_type_entry::<$x>),
                subscribe: Some(subscribe_entry::<$x>),
                advertise: Some(advertise_entry::<$x>),
                get_err_msg: Some(get_err_msg::<$x>),
            };

            ffi::Core{ impl_ptr: self as *mut $x as *mut c_void,
                       vptr: &VTBL as *const ffi::CoreVtbl }
        }
    )
}

pub unsafe extern "C" fn get_type_entry<C: Core>(impl_ptr: *const c_void) -> ffi::StrView {
    assert!(!impl_ptr.is_null());

    let tp = (*(impl_ptr as *const C)).get_type();

    str_to_ffi(tp)
}

pub unsafe extern "C" fn subscribe_entry<C: Core>(
    impl_ptr: *mut c_void, params: ffi::SubscribeParams, subscriber: *mut ffi::Subscriber
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!subscriber.is_null());

    match (*(impl_ptr as *mut C)).subscribe(params) {
        Ok(s) => {
            *subscriber = s.into_ffi();

            0
        },
        Err(e) => e.as_code()
    }
}

pub unsafe extern "C" fn advertise_entry<C: Core>(
    impl_ptr: *mut c_void, params: ffi::AdvertiseParams,
    publisher: *mut ffi::Publisher
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!publisher.is_null());

    match (*(impl_ptr as *mut C)).advertise(params) {
        Ok(p) => {
            *publisher = p.into_ffi();

            0
        },
        Err(e) => e.as_code()
    }
}

pub unsafe extern "C" fn get_err_msg<C: Core>(
    _: *const c_void, err: c_int
) -> ffi::StrView {
    let msg = C::Error::from_code(err).what();

    ffi::StrView{ data: msg.as_ptr() as *const c_char, len: msg.len() as ptrdiff_t }
}

pub mod subscriber {

}

pub mod publisher {

use super::*;

use capnp::message::{Allocator, Builder};
use libc::{c_int, c_void};

pub unsafe extern "C" fn get_channel_name_entry<P: Publisher>(
    impl_ptr: *const c_void
) -> ffi::StrView {
    assert!(!impl_ptr.is_null());

    let name = (*(impl_ptr as *const P)).get_channel_name();

    str_to_ffi(name)
}

pub unsafe extern "C" fn get_channel_type_entry<P: Publisher>(impl_ptr: *const c_void) -> u64 {
    assert!(!impl_ptr.is_null());

    (*(impl_ptr as *const P)).get_channel_type()
}

pub unsafe extern "C" fn publish_entry<P: Publisher>(impl_ptr: *mut c_void,
                                                     publish_fn: Option<ffi::PublishFn>,
                                                     arg: *mut c_void) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(publish_fn.is_some());

    let publisher = &mut *(impl_ptr as *mut P);
    let builder
}

unsafe fn builder_vtbl<A: Allocator>() -> ffi::MsgBuilderVtbl {

}   

// pub unsafe extern "C" fn disconnect_entry(&mut self) -> Result<(), Self::Err> {}

} // pub mod publisher
