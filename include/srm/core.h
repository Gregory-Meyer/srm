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

#include <assert.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef int (*SrmSubscriberCallback)(void *arg, SrmMsgView message);

typedef struct SrmSubscriberParams {
    SrmStrView type;
    SrmStrView topic;
    SrmSubscriberCallback callback;
    void *arg;
} SrmSubscribeParams;

typedef struct SrmCore {
    void *impl;
    int (*subscribe_fn)(void *core, const SrmSubscriberParams *params);
    int (*alloc_msg_fn)(void *core, SrmMsg *message);
    int (*dealloc_msg_fn)(void *core, SrmMsg *message);
    int (*publish_fn)(void *core, SrmMsg *message);
} SrmCore;

inline int srm_Core_subscribe(SrmCore *core, const SrmSubscriberParams *params) {
    assert(core);
    assert(params);
    assert(params->callback);
    assert(params->type.data);
    assert(params->topic.data);

    return core->subscribe_fn(core->impl, params);
}

inline int srm_Core_alloc_msg(SrmCore *core, SrmMsg *msg) {
    assert(core);
    assert(msg);
    assert(msg->type.data);

    return core->alloc_msg_fn(core->impl, msg);
}

inline int srm_Core_dealloc_msg(SrmCore *core, SrmMsg *msg) {
    assert(core);
    assert(msg);
    assert(msg->type.data);

    return core->dealloc_msg_fn(core->impl, msg);
}

inline int srm_Core_publish(SrmCore *core, SrmMsg *msg) {
    assert(core);
    assert(msg);
    assert(msg->type.data);
    assert(msg->topic.data);
    assert(msg->data);

    return core->publish_fn(core->impl, msg);
}

#ifdef __cplusplus
} // extern "C"
#endif

#endif
