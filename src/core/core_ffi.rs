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

use libc::c_void;

pub unsafe extern "C" fn get_type_entry<C: Core>(impl_ptr: *const c_void) -> ffi::StrView {
    assert!(!impl_ptr.is_null());

    let tp = (*(impl_ptr as *const C)).get_type();

    str_to_ffi(tp)
}

pub unsafe extern "C" fn subscribe_entry<C: Core>(
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

pub unsafe extern "C" fn advertise_entry<C: Core>(
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
        len: msg.len() as ptrdiff_t,
    }
}

pub unsafe extern "C" fn log_error_entry<C: Core>(
    impl_ptr: *const c_void,
    msg: ffi::StrView,
) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).log_error(ffi_to_str(msg).unwrap()) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn log_warn_entry<C: Core>(
    impl_ptr: *const c_void,
    msg: ffi::StrView,
) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).log_warn(ffi_to_str(msg).unwrap()) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn log_info_entry<C: Core>(
    impl_ptr: *const c_void,
    msg: ffi::StrView,
) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).log_info(ffi_to_str(msg).unwrap()) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn log_debug_entry<C: Core>(
    impl_ptr: *const c_void,
    msg: ffi::StrView,
) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).log_debug(ffi_to_str(msg).unwrap()) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}

pub unsafe extern "C" fn log_trace_entry<C: Core>(
    impl_ptr: *const c_void,
    msg: ffi::StrView,
) -> c_int {
    assert!(!impl_ptr.is_null());

    match (*(impl_ptr as *const C)).log_trace(ffi_to_str(msg).unwrap()) {
        Ok(()) => 0,
        Err(e) => e.as_code(),
    }
}
