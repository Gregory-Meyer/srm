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
extern crate serde;
extern crate serde_yaml;

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

use std::{env, fs, path::{PathBuf}, sync::Arc, thread};

use serde::Deserialize;

fn main() {
    let args: Vec<_> = env::args().collect();
    let graph_pathname = &args[1];
    let graph_string = fs::read_to_string(graph_pathname).expect("couldn't read node graph");
    let graph: NodeGraph = serde_yaml::from_str(&graph_string).expect("couldn't parse node graph");
    let mut core = static_core::StaticCore::new(graph.path);

    for (name, tp) in graph.nodes.into_iter() {
        if let Err(e) = core.add_node(name.0, tp.0) {
            panic!("couldn't add node: {}", e);
        }
    }

    let run_ptr = Arc::new(core);
    let stop_ptr = run_ptr.clone();

    let handle = thread::spawn(move || {
        run_ptr.run();
    });

    ctrlc::set_handler(move || {
        stop_ptr.stop();
    }).expect("couldn't set ^C handler");

    handle.join().unwrap();
}

#[derive(Deserialize)]
struct Name(String);

#[derive(Deserialize)]
struct Type(String);

#[derive(Deserialize)]
struct NodeGraph {
    path: Vec<PathBuf>,
    nodes: Vec<(Name, Type)>,
}
