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

use capnp::Word;
use libc::c_void;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MsgSegment {
    pub data: *mut Word,
    pub len: Index,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MsgSegmentView {
    pub data: *const Word,
    pub len: Index,
}

unsafe impl Send for MsgSegmentView { }

unsafe impl Sync for MsgSegmentView { }

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MsgView {
    pub segments: *const MsgSegmentView,
    pub num_segments: Index,
    pub msg_type: MsgType,
}

unsafe impl Send for MsgView { }

unsafe impl Sync for MsgView { }

#[repr(C)]
#[derive(Debug)]
pub struct MsgBuilder {
    pub impl_ptr: *mut c_void,
    pub vptr: *const MsgBuilderVtbl,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MsgBuilderVtbl {
    pub alloc_segment: Option<unsafe extern "C" fn(*mut c_void, *mut MsgSegment) -> c_int>,
    pub get_err_msg: Option<unsafe extern "C" fn(*const c_void, c_int) -> StrView>,
}
