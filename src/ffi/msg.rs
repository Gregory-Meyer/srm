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

use libc::c_void;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MsgSegment {
    data: *mut Word,
    len: Index,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MsgSegmentView {
    data: *const Word,
    len: Index,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MsgView {
    segments: *const MsgSegmentView,
    num_segments: Index,
    msg_type: MsgType,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MsgBuilder {
    impl_ptr: *mut c_void,
    vptr: *const MsgBuilderVtbl,
}

impl MsgBuilder {
    pub unsafe fn alloc_segment(self, min_len: Index) -> Result<MsgSegment, (c_int, StrView)> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());
        assert!(min_len > 0);

        let mut segment = MsgSegment{ data: ptr::null_mut(), len: min_len };

        let err = ((*self.vptr).alloc_segment.unwrap())(self.impl_ptr, &mut segment);

        if err != 0 {
            Err((err, self.get_err_msg(err)))
        } else {
            Ok(segment)
        }
    }

    pub unsafe fn get_err_msg(self, err: c_int) -> StrView {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());

        ((*self.vptr).get_err_msg.unwrap())(self.impl_ptr, err)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MsgBuilderVtbl {
    alloc_segment: Option<extern "C" fn(*mut c_void, *mut MsgSegment) -> c_int>,
    get_err_msg: Option<extern "C" fn(*const c_void, c_int) -> StrView>,
}

impl MsgBuilderVtbl {
    pub fn is_non_null(self) -> bool {
        self.alloc_segment.is_some() && self.get_err_msg.is_some()
    }
}
