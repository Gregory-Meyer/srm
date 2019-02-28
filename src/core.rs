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

use capnp::message::Allocator;
use libc::{c_char, c_int, ptrdiff_t};

pub trait Core: Send + Sync {
    type Error: Error;
    type Publisher: Publisher;
    type Subscriber: Subscriber;

    fn get_type(&self) -> &str;

    fn subscribe(&self, params: ffi::SubscribeParams) -> Result<Self::Subscriber, Self::Error>;

    fn advertise(&self, params: ffi::AdvertiseParams) -> Result<Self::Publisher, Self::Error>;

    fn as_ffi(&mut self) -> ffi::Core;
}

pub trait Publisher: Send {
    type Builder: MessageBuilder;
    type Error: Error;

    fn get_channel_name(&self) -> &str;

    fn get_channel_type(&self) -> u64;

    fn publish(&mut self, builder: Self::Builder) -> Result<(), Self::Error>;

    fn into_ffi(self) -> ffi::Publisher;

    fn get_allocator(&self) -> Self::Builder;
}

pub trait Subscriber: Send {
    type Error: Error;

    fn get_channel_name(&self) -> &str;

    fn get_channel_type(&self) -> u64;

    fn into_ffi(self) -> ffi::Subscriber;
}

pub trait MessageBuilder: Send + Allocator {
    type Error: Error;

    unsafe fn as_view(&self) -> Vec<ffi::MsgSegmentView>;

    fn as_ffi(&mut self) -> ffi::MsgBuilder;
}

pub trait Error: error::Error {
    fn from_code(code: c_int) -> Self;

    fn as_code(&self) -> c_int;

    fn what(&self) -> &'static str;
}

#[macro_export]
macro_rules! srm_core_impl {
    ($x:ty) => (
        fn as_ffi(&mut self) -> ffi::Core {
            use libc::c_void;

            const VTBL: ffi::CoreVtbl = ffi::CoreVtbl{
                get_type: Some(core::core_ffi::get_type_entry::<$x>),
                subscribe: Some(core::core_ffi::subscribe_entry::<$x>),
                advertise: Some(core::core_ffi::advertise_entry::<$x>),
                get_err_msg: Some(core::core_ffi::get_err_msg::<$x>),
            };

            ffi::Core{ impl_ptr: self as *mut $x as *mut c_void,
                       vptr: &VTBL as *const ffi::CoreVtbl }
        }
    )
}

#[macro_export]
macro_rules! srm_subscriber_impl {
    ($x:ty) => (
        fn into_ffi(self) -> ffi::Subscriber {
            use libc::c_void;

            const VTBL: ffi::SubscriberVtbl = ffi::SubscriberVtbl{
                get_channel_name: Some(core::subscriber_ffi::get_channel_name_entry::<$x>),
                get_channel_type: Some(core::subscriber_ffi::get_channel_type_entry::<$x>),
                disconnect: Some(core::subscriber_ffi::disconnect_entry::<$x>),
                get_err_msg: Some(core::subscriber_ffi::get_err_msg::<$x>),
            };

            ffi::Subscriber{ impl_ptr: Box::into_raw(Box::new(self)) as *mut c_void,
                             vptr: &VTBL as *const ffi::SubscriberVtbl }
        }
    )
}

#[macro_export]
macro_rules! srm_publisher_impl {
    ($x:ty) => (
        fn into_ffi(self) -> ffi::Publisher {
            use libc::c_void;

            const VTBL: ffi::PublisherVtbl = ffi::PublisherVtbl{
                get_channel_name: Some(core::publisher_ffi::get_channel_name_entry::<$x>),
                get_channel_type: Some(core::publisher_ffi::get_channel_type_entry::<$x>),
                disconnect: Some(core::publisher_ffi::disconnect_entry::<$x>),
                publish: Some(core::publisher_ffi::publish_entry::<$x>),
                get_err_msg: Some(core::publisher_ffi::get_err_msg::<$x>),
            };

            let impl_ptr = Box::into_raw(Box::new(self));

            ffi::Publisher{ impl_ptr: impl_ptr as *mut c_void,
                            vptr: &VTBL as *const ffi::PublisherVtbl }
        }
    )
}

#[macro_export]
macro_rules! srm_message_builder_impl {
    ($x:ty) => (
        fn as_ffi(&mut self) -> ffi::MsgBuilder {
            use libc::c_void;

            const VTBL: ffi::MsgBuilderVtbl = ffi::MsgBuilderVtbl{
                alloc_segment: Some(core::message_builder_ffi::alloc_segment_entry::<$x>),
                get_err_msg: Some(core::message_builder_ffi::get_err_msg::<$x>),
            };

            ffi::MsgBuilder{ impl_ptr: self as *mut $x as *mut c_void,
                             vptr: &VTBL as *const ffi::MsgBuilderVtbl }
        }
    )
}

pub mod core_ffi {

use super::*;

use libc::c_void;

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

} // pub mod core_ffi

pub mod subscriber_ffi {

use super::*;

use std::mem;

use libc::{c_int, c_void};

pub unsafe extern "C" fn get_channel_name_entry<S: Subscriber>(impl_ptr: *const c_void)
-> ffi::StrView {
    assert!(!impl_ptr.is_null());

    let name = (*(impl_ptr as *const S)).get_channel_name();

    str_to_ffi(name)
}

pub unsafe extern "C" fn get_channel_type_entry<S: Subscriber>(impl_ptr: *const c_void) -> u64 {
    assert!(!impl_ptr.is_null());

    (*(impl_ptr as *const S)).get_channel_type()
}

pub unsafe extern "C" fn disconnect_entry<S: Subscriber>(impl_ptr: *mut c_void) -> c_int {
    assert!(!impl_ptr.is_null());

    mem::drop(Box::from_raw(impl_ptr as *mut S));

    0
}

pub unsafe extern "C" fn get_err_msg<S: Subscriber>(_: *const c_void, err: c_int)
-> ffi::StrView {
    let err_obj = S::Error::from_code(err);

    str_to_ffi(err_obj.what())
}

} // pub mod subscriber_ffi

pub mod publisher_ffi {

use super::*;

use std::mem;

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
    let mut alloc = publisher.get_allocator();

    let res = (publish_fn.unwrap())(alloc.as_ffi(), arg);

    if res != 0 {
        return -res;
    }

    match publisher.publish(alloc) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn disconnect_entry<P: Publisher>(impl_ptr: *mut c_void) -> c_int {
    mem::drop(Box::from_raw(impl_ptr as *mut P));

    0
}

pub unsafe extern "C" fn get_err_msg<P: Publisher>(_: *const c_void, err: c_int) -> ffi::StrView {
    let err_obj = P::Error::from_code(err);

    str_to_ffi(err_obj.what())
}

} // pub mod publisher_ffi

pub mod message_builder_ffi {

use super::*;

use libc::{c_int, c_void};

pub unsafe extern "C" fn alloc_segment_entry<B: MessageBuilder>(
    impl_ptr: *mut c_void, segment: *mut ffi::MsgSegment
) -> c_int {
    assert!(!segment.is_null());

    let alloc = &mut *(impl_ptr as *mut B);

    let allocd = alloc.allocate_segment((*segment).len as u32);

    (*segment).data = allocd.0;
    (*segment).len = allocd.1 as ffi::Index;

    0
}

pub unsafe extern "C" fn get_err_msg<B: MessageBuilder>(_: *const c_void, err: c_int)
-> ffi::StrView {
    let err_obj = B::Error::from_code(err);

    str_to_ffi(err_obj.what())
}

} // pub mod message_builder_ffi
