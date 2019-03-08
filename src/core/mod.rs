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

use super::ffi;

use std::error;

use capnp::message::Allocator;
use libc::c_int;

pub enum ParamType {
    Integer,
    Boolean,
    Real,
    String,
}

pub trait CoreBase: Send + Sync {
    fn as_ffi(&self) -> ffi::Core;
}

pub trait Core: Send + Sync + CoreBase {
    type Error: Error;
    type Publisher: Publisher;
    type Subscriber: Subscriber;

    fn get_type(&self) -> &str;

    fn subscribe(&self, params: ffi::SubscribeParams) -> Result<Self::Subscriber, Self::Error>;
    fn advertise(&self, params: ffi::AdvertiseParams) -> Result<Self::Publisher, Self::Error>;

    fn log_error(&self, msg: &str) -> Result<(), Self::Error>;
    fn log_warn(&self, msg: &str) -> Result<(), Self::Error>;
    fn log_info(&self, msg: &str) -> Result<(), Self::Error>;
    fn log_debug(&self, msg: &str) -> Result<(), Self::Error>;
    fn log_trace(&self, msg: &str) -> Result<(), Self::Error>;

    fn param_type(&self, key: &str) -> Result<ParamType, Self::Error>;

    fn param_seti(&self, key: &str, value: isize) -> Result<(), Self::Error>;
    fn param_geti(&self, key: &str) -> Result<isize, Self::Error>;
    fn param_swapi(&self, key: &str, value: isize) -> Result<isize, Self::Error>;

    fn param_setb(&self, key: &str, value: bool) -> Result<(), Self::Error>;
    fn param_getb(&self, key: &str) -> Result<bool, Self::Error>;
    fn param_swapb(&self, key: &str, value: bool) -> Result<bool, Self::Error>;

    fn param_setr(&self, key: &str, value: f64) -> Result<(), Self::Error>;
    fn param_getr(&self, key: &str) -> Result<f64, Self::Error>;
    fn param_swapr(&self, key: &str, value: f64) -> Result<f64, Self::Error>;

    fn param_sets(&self, key: &str, value: String) -> Result<(), Self::Error>;
    fn param_gets(&self, key: &str) -> Result<String, Self::Error>;
    fn param_swaps(&self, key: &str, value: String) -> Result<String, Self::Error>;
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
macro_rules! srm_core_base_impl {
    ($x:ty) => (
        fn as_ffi(&self) -> ffi::Core {
            use libc::c_void;

            const VTBL: ffi::CoreVtbl = ffi::CoreVtbl{
                get_type: Some($crate::core::core_ffi::get_type::<$x>),

                subscribe: Some($crate::core::core_ffi::subscribe::<$x>),
                advertise: Some($crate::core::core_ffi::advertise::<$x>),

                get_err_msg: Some($crate::core::core_ffi::get_err_msg::<$x>),

                log_error: Some($crate::core::core_ffi::log_error::<$x>),
                log_warn: Some($crate::core::core_ffi::log_warn::<$x>),
                log_info: Some($crate::core::core_ffi::log_info::<$x>),
                log_debug: Some($crate::core::core_ffi::log_debug::<$x>),
                log_trace: Some($crate::core::core_ffi::log_trace::<$x>),

                param_type: Some($crate::core::core_ffi::param_type::<$x>),

                param_seti: Some($crate::core::core_ffi::param_seti::<$x>),
                param_geti: Some($crate::core::core_ffi::param_geti::<$x>),
                param_swapi: Some($crate::core::core_ffi::param_swapi::<$x>),

                param_setb: Some($crate::core::core_ffi::param_setb::<$x>),
                param_getb: Some($crate::core::core_ffi::param_getb::<$x>),
                param_swapb: Some($crate::core::core_ffi::param_swapb::<$x>),

                param_setr: Some($crate::core::core_ffi::param_setr::<$x>),
                param_getr: Some($crate::core::core_ffi::param_getr::<$x>),
                param_swapr: Some($crate::core::core_ffi::param_swapr::<$x>),

                param_sets: Some($crate::core::core_ffi::param_sets::<$x>),
                param_gets: Some($crate::core::core_ffi::param_gets::<$x>),
                param_swaps: Some($crate::core::core_ffi::param_swaps::<$x>),
            };

            ffi::Core{ impl_ptr: self as *const $x as *const c_void,
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
                get_channel_name: Some($crate::core::subscriber_ffi::get_channel_name::<$x>),
                get_channel_type: Some($crate::core::subscriber_ffi::get_channel_type::<$x>),
                disconnect: Some($crate::core::subscriber_ffi::disconnect::<$x>),
                get_err_msg: Some($crate::core::subscriber_ffi::get_err_msg::<$x>),
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
                get_channel_name: Some($crate::core::publisher_ffi::get_channel_name::<$x>),
                get_channel_type: Some($crate::core::publisher_ffi::get_channel_type::<$x>),
                disconnect: Some($crate::core::publisher_ffi::disconnect::<$x>),
                publish: Some($crate::core::publisher_ffi::publish::<$x>),
                get_err_msg: Some($crate::core::publisher_ffi::get_err_msg::<$x>),
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
                alloc_segment: Some($crate::core::message_builder_ffi::alloc_segment::<$x>),
                get_err_msg: Some($crate::core::message_builder_ffi::get_err_msg::<$x>),
            };

            ffi::MsgBuilder{ impl_ptr: self as *mut $x as *mut c_void,
                             vptr: &VTBL as *const ffi::MsgBuilderVtbl }
        }
    )
}

pub mod core_ffi;

pub mod subscriber_ffi;

pub mod publisher_ffi;

pub mod message_builder_ffi;
