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

use std::{slice, str};

use libc::{c_char, ptrdiff_t};

/// Converts a foreign string slice into a native string slice.
///
/// # Safety
///
/// Similar to `std::slice::from_raw_parts`, there is no guarantee that the provided slice
/// is valid for as many bytes as it claims to be, nor is the inferred lifetime accurate.
///
/// # Panics
///
/// Panics if `raw` does not point to a valid UTF-8 sequence.
pub unsafe fn ffi_to_str<'a>(raw: ffi::StrView) -> Option<&'a str> {
    if raw.data.is_null() {
        assert!(raw.len == 0);

        return None;
    }

    assert!(raw.len > 0);

    let as_slice: &'a [u8] = slice::from_raw_parts(raw.data as *const u8, raw.len as usize);

    match str::from_utf8(as_slice) {
        Ok(s) => Some(s),
        Err(e) => panic!("StrView was not a valid UTF-8 sequence: {}", e),
    }
}

/// Converts a UTF-8 string slice into an FFI-compatible string slice.
///
/// # Safety
///
/// The StrView must not outlive the string.
pub fn str_to_ffi(s: &str) -> ffi::StrView {
    ffi::StrView{
        data: s.as_ptr() as *const c_char,
        len: s.len() as ptrdiff_t,
    }
}
