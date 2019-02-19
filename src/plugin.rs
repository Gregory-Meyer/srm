use super::*;

use std::path::PathBuf;

use fnv::FnvBuildHasher;
use libloading::{Library, Symbol};

use std::collections::HashMap;

pub struct PluginLoader {
    paths: Vec<PathBuf>,
    plugins: HashMap<String, Library, FnvBuildHasher>,
}

#[derive(Debug)]
pub struct NoSuchLibraryError(String);

impl Error for NoSuchLibraryError { }

impl Display for NoSuchLibraryError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "library {} is missing or not formatted correctly", self.0)
    }
}

impl<'a> PluginLoader {
    pub fn new(paths: Vec<PathBuf>) -> PluginLoader {
        PluginLoader{ paths, plugins: HashMap::with_hasher(FnvBuildHasher::default()) }
    }

    pub fn load(&mut self, name: &str) -> Result<&'a ffi::NodeVtbl, NoSuchLibraryError> {
        match self.plugins.get(name) {
            Some(l) => {
                let getter: Symbol<extern "C" fn() -> *const ffi::NodeVtbl> =
                    match unsafe { l.get(b"srm_Node_get_vtbl") } {
                        Ok(g) => g,
                        Err(_) => return Err(NoSuchLibraryError(String::from(name)))
                    };

                match unsafe { getter().as_ref() } {
                    Some(v) => Ok(v),
                    None => Err(NoSuchLibraryError(String::from(name)))
                }
            }
            None => {
                for mut path in self.paths.iter().cloned() {
                    let filename = if cfg!(windows) {
                        format!("SRM.{}.dll", name)
                    } else {
                        format!("libsrm-{}.so", name)
                    };

                    path.push(filename);

                    if let Ok(l) = Library::new(path) {
                        let getter: Symbol<extern "C" fn() -> *const ffi::NodeVtbl> =
                            match unsafe { l.get(b"srm_Node_get_vtbl") } {
                                Ok(g) => g,
                                Err(_) => return Err(NoSuchLibraryError(String::from(name))),
                            };

                        let vtbl: &'a ffi::NodeVtbl = match unsafe { getter().as_ref() } {
                            Some(v) => v,
                            None => return Err(NoSuchLibraryError(String::from(name))),
                        };

                        self.plugins.insert(name.to_string(), l);

                        return Ok(vtbl);
                    }
                }

                Err(NoSuchLibraryError(String::from(name)))
            }
        }
    }
}
