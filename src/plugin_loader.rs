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

use std::path::{Path, PathBuf};

use hashbrown::{HashMap, hash_map::Entry};
use libloading::Library;

pub struct PluginLoader {
    paths: Vec<PathBuf>,
    plugins: HashMap<String, (Box<node_plugin::NodePlugin>, *const node_plugin::NodePlugin)>,
}

impl PluginLoader {
    pub fn new(paths: Vec<PathBuf>) -> PluginLoader {
        PluginLoader{ paths, plugins: HashMap::new() }
    }

    pub fn load(&mut self, name: String) -> Result<&node_plugin::NodePlugin, node_plugin::LoadError> {
        let entry = self.plugins.entry(name);

        match entry {
            Entry::Occupied(e) => Ok(unsafe { & *e.get().1 }),
            Entry::Vacant(e) => {
                let plugin = PluginLoader::do_load(&self.paths, e.key())?;

                let ptr = Box::into_raw(Box::new(plugin));
                e.insert((unsafe { Box::from_raw(ptr) }, ptr));

                // we never delete a NodePlugin until the PluginLoader is dropped, so this is safe
                // since the lifetime of the returned reference is the same as the lifetime of this
                // PluginLoader
                Ok(unsafe { & *ptr })
            }
        }
    }

    fn do_load(paths: &[PathBuf], name: &str)
    -> Result<node_plugin::NodePlugin, node_plugin::LoadError> {
        for pathname in paths.iter().map(|p| make_lib_name(p, name)) {
            let lib = match Library::new(&pathname) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("failed to load library at {}: {}", pathname.display(), e);

                    continue;
                }
            };

            return node_plugin::NodePlugin::new(lib);
        }

        Err(node_plugin::LoadError::NoLibraryFound)
    }
}

unsafe impl Send for PluginLoader { }

unsafe impl Sync for PluginLoader { }

fn make_lib_name<S: AsRef<str>>(dirname: &Path, name: S) -> PathBuf {
    let mut pathname = dirname.to_path_buf();

    let filename: String = if cfg!(windows) {
        format!("srm-{}.dll", name.as_ref())
    } else {
        format!("libsrm-{}.so", name.as_ref())
    };

    pathname.push(&filename);

    pathname
}
