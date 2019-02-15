/*
 *  Copyright 2019 Gregory Meyer
 *
 *  Permission is hereby granted, free of charge, to any person
 *  obtaining a copy of this software and associated documentation
 *  files (the "Software"), to deal in the Software without
 *  restriction, including without limitation the rights to use, copy,
 *  modify, merge, publish, distribute, sublicense, and/or sell copies
 *  of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be
 *  included in all copies or substantial portions of the Software.
 *
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 *  EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 *  MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 *  NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
 *  BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 *  ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 *  CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 *  SOFTWARE.
 */

#include "node_util.h"

#include <stdexcept>

#ifdef SRM_WINDOWS
#include <winbase.h>
#elif defined(SRM_POSIX)
#include <dlfcn.h>
#endif

namespace srm {

static SharedObjectHandle load_so(const char *path) {
#ifdef SRM_WINDOWS
    const HMODULE object = LoadLibraryA(path);

    if (!object) {
        throw std::system_error(static_cast<int>(GetLastError()), platform_category());
    }
#elif defined(SRM_POSIX)
    void *const object = dlopen(path, RTLD_NOW | RTLD_LOCAL);

    if (!object) {
        throw std::runtime_error(dlerror());
    }
#endif

    return object;
}

static void unload_so(SharedObjectHandle object) {
#ifdef SRM_WINDOWS
    if (!FreeLibrary(object)) {
        throw std::system_error(static_cast<int>(GetLastError()), platform_category());
    }
#elif defined(SRM_POSIX)
    if (dlclose(object) == 0) {
        throw std::runtime_error(dlerror());
    }
#endif
}

template <typename S>
S* load_fn_from_so(SharedObjectHandle object, const char *name) {
#ifdef SRM_WINDOWS
    const FARPROC symbol = GetProcAddress(object, name);

    if (!symbol) {
        throw std::system_error(static_cast<int>(GetLastError()), platform_category());
    }
#elif defined(SRM_POSIX)
    void *const symbol = dlsym(object, name);

    if (!symbol) {
        throw std::runtime_error(dlerror());
    }
#endif

    return reinterpret_cast<S*>(symbol);
}

static NodeVtbl generate_vtable(SharedObjectHandle object) {
    NodeVtbl vtbl;

    vtbl.create_fn = load_fn_from_so<void* (SrmCore*)>(object, "srm_Node_create");
    vtbl.destroy_fn = load_fn_from_so<void (SrmCore*, void*)>(object, "srm_Node_destroy");
    vtbl.run_fn = load_fn_from_so<void (SrmCore*, void*)>(object, "srm_Node_run");
    vtbl.stop_fn = load_fn_from_so<void (SrmCore*, void*)>(object, "srm_Node_stop");


    return vtbl;
}

NodeLibrary::NodeLibrary(const char *path) : handle_(load_so(path)) {
    // wait to do it so the destructor will close the shared object
    vtbl_ = generate_vtable(handle_);
}

NodeLibrary::~NodeLibrary() {
    unload_so(handle_);
}

} // namespace srm
