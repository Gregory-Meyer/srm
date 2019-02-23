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

use std::{marker::PhantomData, ptr};

use libc::{c_int, c_void};

/// Wrapper around ffi::Node and ffi::NodeVtbl.
///
/// Implementations of Node must be thread-safe, so all functions take &self.
pub struct Node<'c, 'v: 'c> {
    impl_ptr: *mut c_void,
    vptr: &'v Vtbl,
    phantom: PhantomData<&'c mut c_void>,
}

impl<'c, 'v> Node<'c, 'v> {
    /// Creates a new Node from the provided core and vtable.
    pub fn new<C: Core>(core: &'c mut C, vptr: &'v Vtbl) -> Result<Node<'c, 'v>, ErrorCode<'v>> {
        let mut node = Node{ impl_ptr: ptr::null_mut(), vptr, phantom: PhantomData };

        let err = (node.vptr.create)(core.as_ffi(), &mut node.impl_ptr);
        node.to_result(err).map(|_| node)
    }

    /// Tells the node to begin computation. Will not return until the node shuts down.
    pub fn run(&self) -> Result<(), ErrorCode> {
        let err = (self.vptr.run)(self.impl_ptr);
        self.to_result(err)
    }

    /// Tells the node to stop computation. Should not block.
    pub fn stop(&self) -> Result<(), ErrorCode> {
        let err = (self.vptr.stop)(self.impl_ptr);
        self.to_result(err)
    }

    /// Returns the string type of the node as it indicates.
    pub fn get_type(&self) -> &str {
        unsafe { ffi_to_str((self.vptr.get_type)(self.impl_ptr)).unwrap() }
    }

    /// Returns the error message corresponding to a given error.
    ///
    /// # Panics
    ///
    /// Panics if the implementation does not return a string to explain err.
    pub fn get_err_msg(&self, err: c_int) -> Option<&str> {
        if err == 0 {
            None
        } else {
            unsafe { Some(ffi_to_str((self.vptr.get_err_msg)(self.impl_ptr, err)).unwrap()) }
        }

    }

    fn to_result<'a>(&self, err: c_int) -> Result<(), ErrorCode<'a>> {
        match err {
            0 => Ok(()),
            x => {
                let msg: &'a str =
                    unsafe { ffi_to_str((self.vptr.get_err_msg)(self.impl_ptr, x)).unwrap() };

                Err(ErrorCode::new(x, msg))
            }
        }
    }
}

impl<'c, 'v> Drop for Node<'c, 'v> {
    /// Calls vptr->destroy.
    ///
    /// # Panics
    ///
    /// Panics if the call to vptr.destroy returns nonzero.
    fn drop(&mut self) {
        match (self.vptr.destroy)(self.impl_ptr) {
            0 => return,
            x => panic!("couldn't drop node {:p}: {} ({})", self.impl_ptr,
                        self.get_err_msg(x).unwrap(), x),
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
