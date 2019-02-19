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

use std::{marker::PhantomData, ptr, slice};

use libc::c_void;

#[repr(C)]
#[derive(Debug)]
pub struct MsgSegment<'a> {
    data: *mut Word,
    len: Index,
    phantom: PhantomData<&'a mut Word>,
}

impl<'a> MsgSegment<'a> {
    pub unsafe fn as_slice(self) -> Option<&'a mut [Word]> {
        if self.data.is_null() {
            assert_eq!(self.len, 0);

            None
        } else {
            assert!(self.len > 0);

            Some(slice::from_raw_parts_mut(self.data, self.len as usize))
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct MsgSegmentView<'a> {
    data: *const Word,
    len: Index,
    phantom: PhantomData<&'a Word>,
}

impl<'a> MsgSegmentView<'a> {
    pub unsafe fn as_slice(self) -> Option<&'a [Word]> {
        if self.data.is_null() {
            assert_eq!(self.len, 0);

            None
        } else {
            assert!(self.len > 0);

            Some(slice::from_raw_parts(self.data, self.len as usize))
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MsgView<'a> {
    segments: *const MsgSegmentView<'a>,
    num_segments: Index,
    msg_type: MsgType,
    phantom: PhantomData<&'a MsgSegmentView<'a>>,
}

impl<'a> MsgView<'a> {
    pub unsafe fn as_slice(self) -> Option<(MsgType, &'a [MsgSegmentView<'a>])> {
        if self.segments.is_null() {
            assert_eq!(self.num_segments, 0);

            None
        } else {
            assert!(self.num_segments > 0);

            Some((self.msg_type, slice::from_raw_parts(self.segments, self.num_segments as usize)))
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct MsgBuilder<'a> {
    impl_ptr: *mut c_void,
    vptr: *const MsgBuilderVtbl<'a>,
    phantom: PhantomData<(&'a mut c_void, &'a MsgBuilderVtbl<'a>)>,
}

impl<'a> MsgBuilder<'a> {
    pub unsafe fn alloc_segment(&'a mut self, min_len: Index) -> Result<'a, &'a mut [Word]> {
        assert!(!self.vptr.is_null());
        assert!((*self.vptr).is_non_null());
        assert!(min_len > 0);

        let mut segment = MsgSegment{ data: ptr::null_mut(), len: min_len, phantom: PhantomData };
        let err = ((*self.vptr).alloc_segment.unwrap())(self.impl_ptr, &mut segment);

        match self.get_err_msg(err) {
            None => Ok(segment.as_slice().unwrap()),
            Some(e) => Err(ForeignError::new(err, e)),
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
#[derive(Copy, Clone)]
pub struct MsgBuilderVtbl<'a> {
    alloc_segment: Option<extern "C" fn(*mut c_void, *mut MsgSegment) -> c_int>,
    get_err_msg: Option<extern "C" fn(*const c_void, c_int) -> StrView<'a>>,
}

impl<'a> MsgBuilderVtbl<'a> {
    pub fn is_non_null(self) -> bool {
        self.alloc_segment.is_some() && self.get_err_msg.is_some()
    }
}
