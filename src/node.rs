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

use super::{core::CoreBase, node_plugin::NodePlugin, *};

use std::{
    ptr,
    sync::{Arc, Weak},
};

use libc::{c_int, c_void};

/// Wrapper around ffi::Node and ffi::NodeVtbl.
///
/// Implementations of Node must be thread-safe, so all functions take &self.
pub struct Node {
    core: Weak<dyn CoreBase>,
    plugin: Arc<NodePlugin>,
    name: String,
    impl_ptr: *mut c_void,
}

unsafe impl Send for Node {}

unsafe impl Sync for Node {}

impl Node {
    /// Creates a new Node from the provided core and vtable.
    pub fn new(core: Arc<dyn CoreBase>, plugin: Arc<NodePlugin>, name: String) -> Node {
        Node {
            core: Arc::downgrade(&core),
            plugin,
            name,
            impl_ptr: ptr::null_mut(),
        }
    }

    pub fn start(&mut self) -> Result<(), ErrorCode> {
        assert!(self.impl_ptr.is_null());
        assert!(self.core.upgrade().is_some());

        let err = unsafe {
            (self.plugin.vptr().create)(
                self.core.upgrade().unwrap().as_ffi(),
                str_to_ffi(&self.name),
                &mut self.impl_ptr,
            )
        };
        self.to_result(err)
    }

    /// Tells the node to begin computation. Will not return until the node shuts down.
    pub fn run(&self) -> Result<(), ErrorCode> {
        assert!(self.core.upgrade().is_some());

        let err = unsafe { (self.plugin.vptr().run)(self.impl_ptr) };
        self.to_result(err)
    }

    /// Tells the node to stop computation. Should not block.
    pub fn stop(&self) -> Result<(), ErrorCode> {
        assert!(self.core.upgrade().is_some());

        let err = unsafe { (self.plugin.vptr().stop)(self.impl_ptr) };
        self.to_result(err)
    }

    /// Returns the string type of the node as it indicates.
    pub fn get_type(&self) -> &str {
        assert!(self.core.upgrade().is_some());

        unsafe { ffi_to_str((self.plugin.vptr().get_type)(self.impl_ptr)).unwrap() }
    }

    /// Returns the error message corresponding to a given error.
    ///
    /// # Panics
    ///
    /// Panics if the implementation does not return a string to explain err.
    pub fn get_err_msg(&self, err: c_int) -> Option<&str> {
        assert!(self.core.upgrade().is_some());

        if err == 0 {
            None
        } else {
            let msg = unsafe { (self.plugin.vptr().get_err_msg)(self.impl_ptr, err) };

            unsafe { Some(ffi_to_str(msg).unwrap()) }
        }
    }

    pub fn name(&self) -> &str {
        assert!(self.core.upgrade().is_some());

        &self.name
    }

    fn to_result<'a>(&self, err: c_int) -> Result<(), ErrorCode> {
        assert!(self.core.upgrade().is_some());

        match err {
            0 => Ok(()),
            x => {
                let msg = unsafe {
                    ffi_to_str((self.plugin.vptr().get_err_msg)(self.impl_ptr, x))
                        .unwrap()
                        .to_string()
                };

                Err(ErrorCode::new(x, msg))
            }
        }
    }
}

impl Drop for Node {
    /// Calls vptr->destroy.
    ///
    /// # Panics
    ///
    /// Panics if the call to vptr.destroy returns nonzero.
    fn drop(&mut self) {
        assert!(self.core.upgrade().is_some());

        match unsafe { (self.plugin.vptr().destroy)(self.impl_ptr) } {
            0 => return,
            x => panic!(
                "couldn't drop node {:p}: {} ({})",
                self.impl_ptr,
                self.get_err_msg(x).unwrap(),
                x
            ),
        }
    }
}

/// Identical to ffi::NodeVtbl, but with all members guaranteed non-null.
pub struct Vtbl {
    pub create: unsafe extern "C" fn(ffi::Core, ffi::StrView, *mut *mut c_void) -> c_int,
    pub destroy: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub run: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub stop: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub get_type: unsafe extern "C" fn(*const c_void) -> ffi::StrView,
    pub get_err_msg: unsafe extern "C" fn(*const c_void, c_int) -> ffi::StrView,
}
