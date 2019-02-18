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

#ifndef SRM_NODE_H
#define SRM_NODE_H

#include <srm/types.h>

#ifdef _WIN32
#define SRM_SHARED_OBJECT_EXPORT __declspec(dllexport)
#else
#define SRM_SHARED_OBJECT_EXPORT
#endif

#ifdef __cplusplus
extern "C" {
#endif

struct SrmNodeVtbl {
    int (*create_fn)(SrmCore core, void **node);
    int (*destroy_fn)(SrmCore core, void *node);
    int (*run_fn)(SrmCore core, void *node);
    int (*stop_fn)(SrmCore core, void *node);
    SrmStrView (*err_str_fn)(int err);
};

SRM_SHARED_OBJECT_EXPORT const SrmNodeVtbl* srm_Node_get_vtbl(void);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
