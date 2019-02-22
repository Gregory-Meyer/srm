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

use libc::{c_int, c_void};

/// Wrapper around ffi::Node and ffi::NodeVtbl.
///
/// Implementations of Node must be thread-safe, so all functions take &self.
pub struct Node<'v> {
    impl_ptr: *mut c_void,
    vptr: &'v Vtbl,
}

impl<'v> Node<'v> {
    /// Creates a new Node from the provided core and vtable.
    pub fn new(core: ffi::Core, vptr: &'v Vtbl) -> Result<Node<'v>, ForeignError<'v>> {
        let mut node = Node{ impl_ptr: ptr::null_mut(), vptr };

        let err = (node.vptr.create)(core, &mut node.impl_ptr);
        node.to_result(err).map(|_| node)
    }

    pub fn run(&self) -> Result<(), ForeignError> {
        let err = (self.vptr.run)(self.impl_ptr);
        self.to_result(err)
    }

    pub fn stop(&self) -> Result<(), ForeignError> {
        let err = (self.vptr.stop)(self.impl_ptr);
        self.to_result(err)
    }

    pub fn get_type(&self) -> &str {
        unsafe { ffi_to_str((self.vptr.get_type)(self.impl_ptr)) }
    }

    pub fn get_err_msg(&self, err: c_int) -> &str {
        unsafe { ffi_to_str((self.vptr.get_err_msg)(self.impl_ptr, err)) }
    }

    fn to_result<'a>(&self, err: c_int) -> Result<(), ForeignError<'a>> {
        match err {
            0 => Ok(()),
            x => {
                assert_eq!(self.impl_ptr, ptr::null_mut());
                let msg: &'a str =
                    unsafe { ffi_to_str((self.vptr.get_err_msg)(self.impl_ptr, x)) };

                Err(ForeignError::new(x, msg))
            }
        }
    }
}

impl<'v> Drop for Node<'v> {
    /// Calls vptr->destroy.
    ///
    /// # Panics
    /// Panics if the call to vptr.destroy returns nonzero.
    fn drop(&mut self) {
        match (self.vptr.destroy)(self.impl_ptr) {
            0 => return,
            x => panic!("couldn't drop node {:p}: {} ({})", self.impl_ptr, self.get_err_msg(x), x),
        }
    }
}

/// Identical to ffi::NodeVtbl, but with all members guaranteed non-null.
pub struct Vtbl {
    create: extern "C" fn(ffi::Core, *mut *mut c_void) -> c_int,
    destroy: extern "C" fn(*mut c_void) -> c_int,
    run: extern "C" fn(*mut c_void) -> c_int,
    stop: extern "C" fn(*mut c_void) -> c_int,
    get_type: extern "C" fn(*const c_void) -> ffi::StrView,
    get_err_msg: extern "C" fn(*const c_void, c_int) -> ffi::StrView,
}
