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

use super::{Error, Subscriber};
use crate::{ffi, util};

use std::mem;

use libc::{c_int, c_void};

pub unsafe extern "C" fn get_channel_name_entry<S: Subscriber>(
    impl_ptr: *const c_void,
) -> ffi::StrView {
    assert!(!impl_ptr.is_null());

    let name = (*(impl_ptr as *const S)).get_channel_name();

    util::str_to_ffi(name)
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

pub unsafe extern "C" fn get_err_msg<S: Subscriber>(_: *const c_void, err: c_int) -> ffi::StrView {
    let err_obj = S::Error::from_code(err);

    util::str_to_ffi(err_obj.what())
}
