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

#ifndef SRM_TYPES_H
#define SRM_TYPES_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef int_least64_t SrmMsgType;
typedef int_least64_t SrmWord;
typedef ptrdiff_t SrmIndex;

typedef struct SrmCore SrmCore;

typedef struct SrmStrView SrmStrView;

typedef struct SrmMsgSegment SrmMsgSegment;

typedef struct SrmMsgSegmentView SrmMsgSegmentView;

typedef struct SrmMsgView SrmMsgView;

typedef struct SrmMsgBuilder SrmMsgBuilder;

typedef struct SrmMsgBuilderVtbl SrmMsgBuilderVtbl;

typedef int (*SrmSubscribeCallback)(SrmCore core, SrmMsgView msg, void *arg);
typedef int (*SrmPublishFn)(SrmCore core, SrmMsgBuilder builder, void *arg);

typedef struct SrmSubscriberParams SrmSubscriberParams;
typedef struct SrmPublishParams SrmPublishParams;

typedef struct SrmCoreVtbl SrmCoreVtbl;

typedef struct SrmNodeVtbl SrmNodeVtbl;

#ifdef __cplusplus
} // extern "C"
#endif

#endif
