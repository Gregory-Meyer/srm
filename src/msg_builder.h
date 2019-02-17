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

#ifndef SRM_IMPL_MSG_BUILDER_H
#define SRM_IMPL_MSG_BUILDER_H

#include "word_arr.h"
#include <srm/types.h>

#include <vector>

#include <capnp/message.h>
#include <tbb/cache_aligned_allocator.h>

static_assert(sizeof(SrmWord) == sizeof(capnp::word) && alignof(SrmWord) == alignof(capnp::word));

namespace srm {

/** MsgBuilder allocates cache-aligned message segments. */
class MsgBuilder : public capnp::MessageBuilder {
public:
    virtual ~MsgBuilder() = default;

    /**
     *  @param minimum_size The minimum number of words to allocate.
     *                      The allocated block will round this up to
     *                      the nearest multiple of 16 words (128
     *                      bytes). Must be positive.
     *  @returns A segment of memory that is cache-aligned and a
     *           multiple of the cache line size in length.
     */
    kj::ArrayPtr<capnp::word> allocateSegment(capnp::uint minimum_size) override;

    /**
     *  @returns An SrmMsgBuilder suitable for use by publishing
     *           subroutines. Its lifetime is tied to this MsgBuilder.
     */
    SrmMsgBuilder as_builder() noexcept;

private:
    std::vector<WordArr, tbb::cache_aligned_allocator<WordArr>> segments_;
};

} // namespace srm

#endif
