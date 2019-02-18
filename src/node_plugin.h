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

#ifndef SRM_IMPL_NODE_PLUGIN_H
#define SRM_IMPL_NODE_PLUGIN_H

#include "shared_obj.h"
#include <srm/node.h>

namespace srm {

/**
 *  NodePlugin manages a Node plugin and ensures that its vtable lives
 *  as long as the managed object.
 */
class NodePlugin {
public:
    /**
     *  Loads a plugin from the specified path according to the OS'
     *  lookup rules, resolves the vtable symbol, and loads the
     *  plugin's vtable.
     *
     *  @param filename Must not be NULL and must be null terminated.
     */
    explicit NodePlugin(const char *filename);

    const SrmNodeVtbl& vtbl() const noexcept {
        assert(vtbl_);

        return *vtbl_;
    }

private:
    SharedObj obj_;
    const SrmNodeVtbl *vtbl_;
};

} // namespace srm

#endif
