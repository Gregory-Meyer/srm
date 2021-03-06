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

use super::{Core, Error, ParamType, Publisher, Subscriber};
use crate::{ffi, util};

use std::{mem, ptr};

use libc::{c_char, c_int, c_void};

pub unsafe extern "C" fn get_type<C: Core>(impl_ptr: *const c_void) -> ffi::StrView {
    assert!(!impl_ptr.is_null());

    let tp = (*(impl_ptr as *const C)).get_type();

    util::str_to_ffi(tp)
}

pub unsafe extern "C" fn subscribe<C: Core>(
    impl_ptr: *const c_void,
    params: ffi::SubscribeParams,
    subscriber: *mut ffi::Subscriber,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!subscriber.is_null());

    match (*(impl_ptr as *const C)).subscribe(params) {
        Ok(s) => {
            *subscriber = s.into_ffi();

            0
        }
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn advertise<C: Core>(
    impl_ptr: *const c_void,
    params: ffi::AdvertiseParams,
    publisher: *mut ffi::Publisher,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!publisher.is_null());

    match (*(impl_ptr as *const C)).advertise(params) {
        Ok(p) => {
            *publisher = p.into_ffi();

            0
        }
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn get_err_msg<C: Core>(_: *const c_void, err: c_int) -> ffi::StrView {
    let msg = C::Error::from_code(err).what();

    ffi::StrView {
        data: msg.as_ptr() as *const c_char,
        len: msg.len() as ffi::Index,
    }
}

pub unsafe extern "C" fn log_error<C: Core>(impl_ptr: *const c_void, msg: ffi::StrView) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).log_error(util::ffi_to_str(msg).unwrap()) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn log_warn<C: Core>(impl_ptr: *const c_void, msg: ffi::StrView) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).log_warn(util::ffi_to_str(msg).unwrap()) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn log_info<C: Core>(impl_ptr: *const c_void, msg: ffi::StrView) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).log_info(util::ffi_to_str(msg).unwrap()) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn log_debug<C: Core>(impl_ptr: *const c_void, msg: ffi::StrView) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).log_debug(util::ffi_to_str(msg).unwrap()) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn log_trace<C: Core>(impl_ptr: *const c_void, msg: ffi::StrView) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).log_trace(util::ffi_to_str(msg).unwrap()) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_type<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    tp: *mut c_int,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!tp.is_null());

    match (*(impl_ptr as *const C)).param_type(util::ffi_to_str(key).unwrap()) {
        Ok(t) => {
            *tp = match t {
                ParamType::Integer => ffi::ParamType::SRM_INTEGER as c_int,
                ParamType::Boolean => ffi::ParamType::SRM_BOOLEAN as c_int,
                ParamType::Real => ffi::ParamType::SRM_REAL as c_int,
                ParamType::String => ffi::ParamType::SRM_STRING as c_int,
            };

            0
        }
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_seti<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    value: isize,
) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).param_seti(util::ffi_to_str(key).unwrap(), value) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_geti<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    result: *mut isize,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!result.is_null());

    match (*(impl_ptr as *const C)).param_geti(util::ffi_to_str(key).unwrap()) {
        Ok(v) => {
            *result = v;

            0
        }
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_swapi<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    value: isize,
    result: *mut isize,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!result.is_null());

    match (*(impl_ptr as *const C)).param_swapi(util::ffi_to_str(key).unwrap(), value) {
        Ok(v) => {
            *result = v;

            0
        }
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_setb<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    value: c_int,
) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).param_setb(util::ffi_to_str(key).unwrap(), value != 0) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_getb<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    result: *mut c_int,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!result.is_null());

    match (*(impl_ptr as *const C)).param_getb(util::ffi_to_str(key).unwrap()) {
        Ok(v) => {
            *result = v as c_int; // 0 is false, 1 is true when casting bool to integral types

            0
        }
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_swapb<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    value: c_int,
    result: *mut c_int,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!result.is_null());

    match (*(impl_ptr as *const C)).param_swapb(util::ffi_to_str(key).unwrap(), value != 0) {
        Ok(v) => {
            *result = v as c_int;

            0
        }
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_setr<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    value: f64,
) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).param_setr(util::ffi_to_str(key).unwrap(), value) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_getr<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    result: *mut f64,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!result.is_null());

    match (*(impl_ptr as *const C)).param_getr(util::ffi_to_str(key).unwrap()) {
        Ok(v) => {
            *result = v;

            0
        }
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_swapr<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    value: f64,
    result: *mut f64,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!result.is_null());

    match (*(impl_ptr as *const C)).param_swapr(util::ffi_to_str(key).unwrap(), value) {
        Ok(v) => {
            *result = v;

            0
        }
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_sets<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    value: ffi::StrView,
) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).param_sets(
        util::ffi_to_str(key).unwrap(),
        util::ffi_to_str(value).unwrap().to_string(),
    ) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_gets<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    result: *mut ffi::String,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!result.is_null());

    match (*(impl_ptr as *const C)).param_gets(util::ffi_to_str(key).unwrap()) {
        Ok(v) => {
            *result = string_to_ffi(v);

            0
        }
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn param_swaps<C: Core>(
    impl_ptr: *const c_void,
    key: ffi::StrView,
    value: ffi::StrView,
    result: *mut ffi::String,
) -> c_int {
    assert!(!impl_ptr.is_null());
    assert!(!result.is_null());

    match (*(impl_ptr as *const C)).param_swaps(
        util::ffi_to_str(key).unwrap(),
        util::ffi_to_str(value).unwrap().to_string(),
    ) {
        Ok(v) => {
            *result = string_to_ffi(v);

            0
        }
        Err(e) => e.as_code(),
    }
}

unsafe extern "C" fn drop_string(data: *mut c_char, capacity: ffi::Index, _: *mut c_void) {
    mem::drop(Vec::from_raw_parts(
        data,
        capacity as usize,
        capacity as usize,
    ));
}

unsafe fn string_to_ffi(mut s: String) -> ffi::String {
    let data = s.as_mut_vec().as_mut_ptr() as *mut c_char;
    let len = s.len() as ffi::Index;
    let capacity = s.capacity() as ffi::Index;
    mem::forget(s);

    ffi::String {
        data,
        len,
        capacity,
        drop_arg: ptr::null_mut(),
        drop: Some(drop_string),
    }
}
