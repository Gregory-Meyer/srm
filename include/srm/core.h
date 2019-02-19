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

struct SrmSubscribeParams {
    SrmMsgType type;
    SrmStrView topic;
    SrmSubscribeCallback callback;
    void *arg;
};

struct SrmAdvertiseParams {
    SrmMsgType type;
    SrmStrView topic;
    SrmPublishFn fn;
    void *arg;
};

struct SrmSubscriberVtbl {
    SrmStrView (*get_channel_name)(const void *impl_ptr);
    SrmMsgType (*get_channel_type)(const void *impl_ptr);
    int (*disconnect)(void *impl_ptr);
    SrmStrView (*get_err_msg)(const void *impl_ptr, int err);
};

struct SrmSubscriber {
    void *impl_ptr;
    const SrmSubscriberVtbl *vptr;
};

struct SrmPublisherVtbl {
    SrmStrView (*get_channel_name)(const void *impl_ptr);
    SrmMsgType (*get_channel_type)(const void *impl_ptr);
    int (*publish)(void *impl_ptr, SrmPublishFn fn, void *arg);
    int (*disconnect)(void *impl_ptr);
    SrmStrView (*get_err_msg)(const void *impl_ptr, int err);
};

struct SrmPublisher {
    void *impl_ptr;
    const SrmPublisherVtbl *vptr;
};

struct SrmCoreVtbl {
    SrmStrView (*get_type)(const void *impl_ptr);
    int (*subscribe)(void *impl_ptr, SrmSubscribeParams params, SrmSubscriber *subscriber);
    int (*advertise)(void *impl_ptr, SrmPublishParams params, SrmPublisher *publisher);
    SrmStrView (*get_err_msg)(const void *impl_ptr, int err);
};

#ifdef __cplusplus
} // extern "C"
#endif

#endif
