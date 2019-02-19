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
use plugin::PluginLoader;

use std::collections::{HashMap, hash_map::Entry};

use libc::{c_int, c_void};
use fnv::FnvBuildHasher;

pub struct Core {
    plugins: PluginLoader,
    nodes: HashMap<String, ffi::Node, FnvBuildHasher>,
}

impl<'a> Core {
    pub fn new(node_path: Vec<PathBuf>) -> Core {
        Core{ plugins: PluginLoader::new(node_path),
              nodes: HashMap::with_hasher(FnvBuildHasher::default()) }
    }

    pub fn add_node(&mut self, name: String) -> Result<&'a mut ffi::Node, AddNodeError> {
        match self.nodes.entry(name) {
            Entry::Occupied(_) => Err(AddNodeError::Duplicate),
            Entry::Vacant(e) => {
                let vtbl = match self.plugins.load(&name) {
                    Ok(v) => v,
                    Err(_) => return Err(AddNodeError::LibraryError),
                };

                let node: ffi::Node = unimplemented!();

                match e.insert(node) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(AddNodeError::LibraryError),
                }
            }
        }
    }
}

impl Core {
    extern "C" fn get_type_entry(_impl_ptr: *const c_void) -> ffi::StrView {
        unimplemented!()
    }

    extern "C" fn subscribe_entry(_impl_ptr: *mut c_void, _params: ffi::SubscribeParams,
                                  _subscriber: *mut ffi::Subscriber) -> c_int {
        unimplemented!()
    }

    extern "C" fn advertise_entry(_impl_ptr: *mut c_void, _params: ffi::AdvertiseParams,
                                  _publisher: *mut ffi::Publisher) -> c_int {
        unimplemented!()
    }

    extern "C" fn get_err_msg_entry(_impl_ptr: *const c_void, _err: c_int) -> ffi::StrView {
        unimplemented!()
    }

    const VTBL: ffi::CoreVtbl = ffi::CoreVtbl{
        get_type: Some(Core::get_type_entry),
        subscribe: Some(Core::subscribe_entry),
        advertise: Some(Core::advertise_entry),
        get_err_msg: Some(Core::get_err_msg_entry),
    };
}

pub trait SerializableError {
    fn errc(&self) -> c_int;
}

pub enum AddNodeError {
    Duplicate,
    LibraryError,
    CreateError(ffi::ForeignError<'static>),
}
