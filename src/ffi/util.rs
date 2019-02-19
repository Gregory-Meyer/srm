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

use std::{slice, str};

use std::marker::PhantomData;

use libc::{c_char, ptrdiff_t};

#[repr(C)]
#[derive(Debug)]
pub struct StrView<'a> {
    data: *const c_char,
    len: ptrdiff_t,
    phantom: PhantomData<&'a c_char>,
}

impl<'a> StrView<'a> {
    pub unsafe fn into_str(self) -> Option<&'a str> {
        if self.data.is_null() {
            assert!(self.len == 0);

            None
        } else {
            assert!(self.len > 0);

            let slice = slice::from_raw_parts(self.data as *const u8, self.len as usize);

            Some(str::from_utf8(&slice).unwrap())
        }
    }

    pub fn from_str(s: &'a str) -> StrView<'a> {
        let bytes = s.as_bytes();

        StrView{ data: bytes.as_ptr() as *const c_char, len: s.len() as ptrdiff_t,
                 phantom: PhantomData }
    }
}
