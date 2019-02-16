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

#ifndef SRM_MSG_H
#define SRM_MSG_H

#include <srm/types.h>
#include <srm/util.h>

#include <assert.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

struct SrmMsgSegment {
    SrmWord *data;
    SrmIndex len;
};

struct SrmMsgSegmentView {
    const SrmWord *data;
    SrmIndex len;
};

struct SrmMsgView {
    const SrmMsgSegmentView *segments;
    SrmIndex num_segments;
    SrmMsgType type;
};

struct SrmMsgAlloc {
    void *impl_ptr;
    const SrmMsgAllocVtbl *vtbl;
};

struct SrmMsgAllocVtbl {
    int (*alloc_segment)(void *impl_ptr, SrmMsgSegment *segment);
    int (*dealloc_segment)(void *impl_ptr, SrmMsgSegment segment);
    SrmStrView (*err_to_str)(int err);
};

inline int srm_MsgAlloc_alloc_segment(SrmMsgAlloc *alloc, SrmMsgSegment *segment) {
    assert(alloc);
    assert(alloc->impl_ptr);
    assert(alloc->vtbl);
    assert(alloc->vtbl->alloc_segment);
    assert(alloc->vtbl->dealloc_segment);
    assert(alloc->vtbl->err_to_str);
    assert(segment);
    assert(segment->len > 0);

    return alloc->vtbl->alloc_segment(alloc->impl_ptr, segment);
}

inline int srm_MsgAlloc_dealloc_segment(SrmMsgAlloc *alloc, SrmMsgSegment segment) {
    assert(alloc);
    assert(alloc->impl_ptr);
    assert(alloc->vtbl);
    assert(alloc->vtbl->alloc_segment);
    assert(alloc->vtbl->dealloc_segment);
    assert(alloc->vtbl->err_to_str);
    assert(segment.data);
    assert(segment.len > 0);

    return alloc->vtbl->dealloc_segment(alloc->impl_ptr, segment);
}

inline SrmStrView srm_MsgAlloc_err_to_str(const SrmMsgAlloc *alloc, int err) {
    assert(alloc);
    assert(alloc->impl_ptr);
    assert(alloc->vtbl);
    assert(alloc->vtbl->alloc_segment);
    assert(alloc->vtbl->dealloc_segment);
    assert(alloc->vtbl->err_to_str);

    return alloc->vtbl->err_to_str(err);
}

#ifdef __cplusplus
} // extern "C"
#endif

#endif
