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

#ifndef SRM_IMPL_NODE_UTIL_H
#define SRM_IMPL_NODE_UTIL_H

#include <srm/core.h>

#include "plat.h"

namespace srm {

#ifdef SRM_WINDOWS
using SharedObjectHandle = HMODULE;
#elif defined(SRM_POSIX)
using SharedObjectHandle = void*;
#endif

using NodeCreateFn = void* (*)(SrmCore *core);
using NodeDestroyFn = void (*)(SrmCore *core, void *node);
using NodeRunFn = void (*)(SrmCore *core, void *node);
using NodeStopFn = void (*)(SrmCore *core, void *node);

struct NodeVtbl {
    NodeCreateFn create_fn;
    NodeDestroyFn destroy_fn;
    NodeRunFn run_fn;
    NodeStopFn stop_fn;
};

class NodeLibrary {
public:
    explicit NodeLibrary(const char *path);

    ~NodeLibrary();

    constexpr const NodeVtbl& vtbl() const noexcept {
        return vtbl_;
    }

private:
    SharedObjectHandle handle_;
    NodeVtbl vtbl_;
};

} // namespace srm

#endif
