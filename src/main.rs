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

extern crate capnp;
extern crate ctrlc;
extern crate env_logger;
extern crate hashbrown;
extern crate libc;
extern crate libloading;
extern crate lock_api;
extern crate log;
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

use std::{env, fs, path::PathBuf, sync::Arc};

use serde::Deserialize;

fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().collect();
    let graph_pathname = &args[1];
    let graph_string = fs::read_to_string(graph_pathname).expect("couldn't read node graph");
    let graph: NodeGraph = serde_yaml::from_str(&graph_string).expect("couldn't parse node graph");
    let core = Arc::new(static_core::Core::new(graph.path));

    for (name, tp) in graph.nodes.into_iter() {
        if let Err(e) = static_core::add_node(core.clone(), name.0, tp.0) {
            panic!("couldn't add node: {}", e);
        }
    }

    let stop_ptr = core.clone();

    ctrlc::set_handler(move || {
        stop_ptr.stop();
    })
    .expect("couldn't set ^C handler");

    core.run();
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
