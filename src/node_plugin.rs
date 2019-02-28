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

use std::{error::Error, fmt::{self, Display, Formatter}};

use libloading::Library;

pub struct NodePlugin {
    library: Library,
    vtbl: node::Vtbl,
}

impl NodePlugin {
    pub fn new(library: Library) -> Result<NodePlugin, LoadError> {
        let f = unsafe { library.get::<GetVtblFn>(b"srm_Node_get_vtbl\0") }
            .map_err(|_| LoadError::LibraryMissingSymbol)?;
        let vptr = unsafe { f().as_ref() }.ok_or(LoadError::VtblNull)?;

        if vptr.create.is_none() {
            return Err(LoadError::VtblMissingFunction("create"));
        } else if vptr.destroy.is_none() {
            return Err(LoadError::VtblMissingFunction("destroy"));
        } else if vptr.run.is_none() {
            return Err(LoadError::VtblMissingFunction("run"));
        } else if vptr.stop.is_none() {
            return Err(LoadError::VtblMissingFunction("stop"));
        } else if vptr.get_type.is_none() {
            return Err(LoadError::VtblMissingFunction("get_type"));
        } else if vptr.get_err_msg.is_none() {
            return Err(LoadError::VtblMissingFunction("get_err_msg"));
        }

        Ok(NodePlugin{
            library,
            vtbl: node::Vtbl{
                create: vptr.create.unwrap(),
                destroy: vptr.destroy.unwrap(),
                run: vptr.run.unwrap(),
                stop: vptr.stop.unwrap(),
                get_type: vptr.get_type.unwrap(),
                get_err_msg: vptr.get_err_msg.unwrap(),
            }
        })
    }

    pub fn vptr(&self) -> &node::Vtbl {
        &self.vtbl
    }
}

unsafe impl Send for NodePlugin { }

unsafe impl Sync for NodePlugin { }

type GetVtblFn = unsafe extern "C" fn() -> *const ffi::NodeVtbl;

#[derive(Debug)]
pub enum LoadError {
    NoLibraryFound,
    LibraryMissingSymbol,
    VtblNull,
    VtblMissingFunction(&'static str),
}

impl Error for LoadError { }

impl Display for LoadError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LoadError::NoLibraryFound
                => write!(f, "no library found in search path"),
            LoadError::LibraryMissingSymbol
                => write!(f, "library missing symbol 'srm_Node_get_vtbl'"),
            LoadError::VtblNull
                => write!(f, "'srm_Node_get_vtbl' returned NULL"),
            LoadError::VtblMissingFunction(name)
                => write!(f, "vtbl missing function '{}'", name),
        }
    }
}
