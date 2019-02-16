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

#ifndef SRM_CORE_H
#define SRM_CORE_H

#include <srm/msg.h>
#include <srm/util.h>
#include <srm/types.h>

#include <assert.h>

#ifdef __cplusplus
extern "C" {
#endif

struct SrmCore {
    void *impl_ptr;
    const SrmCoreVtbl *vtbl;
};

struct SrmSubscriberParams {
    SrmMsgType type;
    SrmStrView topic;
    SrmSubscribeCallback callback;
    void *arg;
};

struct SrmCoreVtbl {
    int (*subscribe)(void *impl_ptr, SrmSubscriberParams params);
    int (*publish)(void *impl_ptr, SrmPublishFn fn, SrmCore *core, void *arg);
    SrmStrView (*err_to_str)(int err);
};

inline int srm_Core_subscribe(SrmCore *core, SrmSubscriberParams params) {
    assert(core);
    assert(core->impl_ptr);
    assert(core->vtbl);
    assert(core->vtbl->subscribe);
    assert(core->vtbl->publish);
    assert(core->vtbl->err_to_str);
    assert(params.type & (SrmMsgType) 1 << 63);
    assert(params.topic.data);
    assert(params.callback);

    return core->vtbl->subscribe(core->impl_ptr, params);
}

inline int srm_Core_publish(SrmCore *core, SrmPublishFn fn, void *arg) {
    assert(core);
    assert(core->impl_ptr);
    assert(core->vtbl);
    assert(core->vtbl->subscribe);
    assert(core->vtbl->publish);
    assert(core->vtbl->err_to_str);
    assert(fn);

    return core->vtbl->publish(core->impl_ptr, fn, core, arg);
}

inline SrmStrView srm_Core_err_to_str(const SrmCore *core, int err) {
    assert(core);
    assert(core->impl_ptr);
    assert(core->vtbl);
    assert(core->vtbl->subscribe);
    assert(core->vtbl->publish);
    assert(core->vtbl->err_to_str);

    return core->vtbl->err_to_str(err);
}

#ifdef __cplusplus
} // extern "C"
#endif

#endif
