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
extern crate hashbrown;
extern crate humantime;
extern crate libc;
extern crate libloading;
extern crate lock_api;
extern crate log;
extern crate parking_lot;
extern crate rayon;
extern crate serde;
extern crate serde_yaml;

mod alloc;
mod core;
mod error_code;
mod ffi;
mod logging;
mod node;
mod node_graph;
mod node_plugin;
mod plugin_loader;
mod static_core;
mod util;

use std::process;

use log::error;

fn main() {
    logging::init();

    let core = match node_graph::spawn_core() {
        Ok(c) => c,
        Err(e) => {
            error!("couldn't spawn core from node graph: {}", e);
            log::logger().flush();

            process::exit(1);
        }
    };

    let other_core = core.clone();

    match ctrlc::set_handler(move || {
        other_core.stop();
    }) {
        Ok(_) => (),
        Err(e) => {
            error!("couldn't set ^C handler: {}", e);
            log::logger().flush();

            process::exit(1);
        }
    };

    core.run();
    log::logger().flush();
}
