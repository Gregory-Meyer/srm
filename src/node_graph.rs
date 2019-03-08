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

use crate::static_core::{self, NodeError, StaticCore};

use std::{
    env,
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, Read},
    path::PathBuf,
    sync::Arc,
};

use hashbrown::HashSet;
use log::info;
use serde::Deserialize;

pub fn spawn_core() -> Result<Arc<StaticCore>, GraphError> {
    let graph = if let Some(filename) = env::args_os().nth(1) {
        if filename == "-" {
            info!("reading node graph from stdin");
            NodeGraph::from_reader(&mut io::stdin())?
        } else {
            info!("reading node graph from '{}'", filename.to_string_lossy());
            let mut file = File::open(&filename).map_err(|e| GraphError::File(e))?;

            NodeGraph::from_reader(&mut file)?
        }
    } else {
        info!("reading node graph from stdin");
        NodeGraph::from_reader(&mut io::stdin())?
    };

    graph.into_static_core().map_err(|e| GraphError::Node(e))
}

#[derive(Deserialize)]
struct NodeGraph {
    path: Vec<PathBuf>,
    nodes: Vec<(String, String)>, // (name, type)
}

impl NodeGraph {
    fn from_reader<R: Read>(reader: &mut R) -> Result<NodeGraph, GraphError> {
        let mut buf = String::new();

        reader
            .read_to_string(&mut buf)
            .map_err(|e| GraphError::Input(e))?;

        let graph: NodeGraph =
            serde_yaml::from_str(&buf).map_err(|e| GraphError::Deserialize(e))?;

        {
            let mut names = HashSet::new();
            for name in graph.nodes.iter().map(|(n, _)| n.as_str()) {
                if !names.insert(name) {
                    return Err(GraphError::DuplicateName(name.to_string()));
                }
            }
        }

        Ok(graph)
    }

    fn into_static_core(self) -> Result<Arc<StaticCore>, NodeError> {
        let core = Arc::new(StaticCore::new(self.path));

        for (name, tp) in self.nodes.into_iter() {
            static_core::add_node(&core, name, tp)?;
        }

        Ok(core)
    }
}

#[derive(Debug)]
pub enum GraphError {
    File(io::Error),
    Input(io::Error),
    Deserialize(serde_yaml::Error),
    DuplicateName(String),
    Node(NodeError),
}

impl Error for GraphError {}

impl Display for GraphError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GraphError::File(e) => write!(f, "couldn't open file for reading: {}", e),
            GraphError::Input(e) => write!(f, "couldn't read from file: {}", e),
            GraphError::Deserialize(e) => write!(f, "input wasn't valid YAML: {}", e),
            GraphError::DuplicateName(n) => {
                write!(f, "input contained duplicate node name '{}'", n)
            }
            GraphError::Node(e) => write!(f, "couldn't initialize core from graph: {}", e),
        }
    }
}
