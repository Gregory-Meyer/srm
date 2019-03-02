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

use std::{
    alloc::{self, Layout},
    error::Error,
    fmt::{self, Display, Formatter},
    mem,
};

use capnp::message::{Allocator, Builder};
use libc::c_int;

/// Cache aligned message builder.
pub type CacheAlignedBuilder = Builder<CacheAlignedAllocator>;

/// Allocates cache-aligned message segments.
pub struct CacheAlignedAllocator {
    segments: Vec<(*mut capnp::Word, usize)>,
}

impl CacheAlignedAllocator {
    pub fn new() -> CacheAlignedAllocator {
        CacheAlignedAllocator {
            segments: Vec::new(),
        }
    }
}

unsafe impl Send for CacheAlignedAllocator {}

unsafe impl Sync for CacheAlignedAllocator {}

unsafe impl Allocator for CacheAlignedAllocator {
    /// Allocates segments that are multiples of 16 words (128 bytes) long.
    ///
    /// Messages are allocated using `std::alloc::alloc_zeroed`.
    fn allocate_segment(&mut self, min_num_words: u32) -> (*mut capnp::Word, u32) {
        let (buf, sz) = alloc_at_least(min_num_words as usize);
        self.segments.push((buf, sz));

        (buf, sz as u32)
    }
}

impl core::MessageBuilder for CacheAlignedAllocator {
    type Error = NullError;

    unsafe fn as_view(&self) -> Vec<ffi::MsgSegmentView> {
        self.segments
            .iter()
            .map(|(p, l)| ffi::MsgSegmentView {
                data: *p,
                len: *l as ffi::Index,
            })
            .collect()
    }

    srm_message_builder_impl!(CacheAlignedAllocator);
}

impl Drop for CacheAlignedAllocator {
    /// Deallocates all allocated message segments.
    fn drop(&mut self) {
        for (buf, sz) in self.segments.iter() {
            let num_bytes = sz * mem::size_of::<capnp::Word>();
            let layout = unsafe { Layout::from_size_align_unchecked(num_bytes, CACHE_SIZE) };

            unsafe { alloc::dealloc(*buf as *mut u8, layout) };
        }
    }
}

#[derive(Debug)]
pub struct NullError();

impl core::Error for NullError {
    fn from_code(_: c_int) -> NullError {
        NullError()
    }

    fn as_code(&self) -> c_int {
        0
    }

    fn what(&self) -> &'static str {
        "ok"
    }
}

impl Error for NullError {}

impl Display for NullError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "NullError")
    }
}

// a smarter man would use cpuid, but this is good enough for TBB
const CACHE_SIZE: usize = 128;

fn alloc_at_least(min_num_words: usize) -> (*mut capnp::Word, usize) {
    let num_bytes =
        round_up_to_nearest_multiple_of_cache_size(mem::size_of::<capnp::Word>() * min_num_words);
    let num_words = num_bytes / mem::size_of::<capnp::Word>();

    let layout = unsafe { Layout::from_size_align_unchecked(num_bytes, CACHE_SIZE) };
    let buf = unsafe { alloc::alloc_zeroed(layout) } as *mut capnp::Word;

    if buf.is_null() {
        alloc::handle_alloc_error(layout);
    }

    (buf, num_words)
}

fn round_up_to_nearest_multiple_of_cache_size(x: usize) -> usize {
    if x % CACHE_SIZE != 0 {
        x + CACHE_SIZE - x % 128
    } else {
        x
    }
}
