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

#![allow(dead_code)]

extern crate capnp;
extern crate ctrlc;
extern crate fnv;
extern crate libc;
extern crate libloading;
extern crate lock_api;
extern crate parking_lot;
extern crate rayon;

pub mod alloc;
pub mod core;
pub mod error_code;
pub mod ffi;
pub mod node;
pub mod node_plugin;
pub mod plugin_loader;
pub mod static_core;
pub mod util;

pub use self::error_code::*;
pub use self::util::*;

use std::{path::{PathBuf}, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread};

fn main() {
    let mut paths = Vec::new();
    paths.push(PathBuf::from("."));
    paths.push(PathBuf::from("/usr/local/lib"));
    paths.push(PathBuf::from("/usr/lib"));

    let mut core = static_core::StaticCore::new(paths);

    core.add_node("publisher".to_string(), "publisher".to_string()).unwrap();
    core.add_node("subscriber".to_string(), "subscriber".to_string()).unwrap();

    let run_ptr = Arc::new(core);
    let stop_ptr = run_ptr.clone();

    let keep_running = AtomicBool::new(true);

    thread::spawn(move || {
        run_ptr.run();
    });

    ctrlc::set_handler(move || {
        stop_ptr.stop();
    }).unwrap();

    while keep_running.load(Ordering::SeqCst) { }
}
