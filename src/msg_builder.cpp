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

#include "msg_builder.h"

#include <srm/msg.h>

#include <string_view>

namespace srm {

/**
 *  @param minimum_size The minimum number of words to allocate.
 *                      The allocated block will round this up to
 *                      the nearest multiple of 16 words (128
 *                      bytes). Must be positive.
 *  @returns A segment of memory that is cache-aligned and a
 *           multiple of the cache line size in length.
 */
kj::ArrayPtr<capnp::word> MsgBuilder::allocateSegment(capnp::uint minimum_size) {
    if (minimum_size % 128 != 0) {
        minimum_size += (128 - minimum_size % 128);
    }

    WordArr &arr = segments_.emplace_back(minimum_size);

    return {arr.data(), arr.size()};
}

static const SrmMsgBuilderVtbl& get_vtbl() noexcept;

/**
 *  @returns An SrmMsgBuilder suitable for use by publishing
 *           subroutines. Its lifetime is tied to this MsgBuilder.
 */
SrmMsgBuilder MsgBuilder::as_builder() noexcept {
    return {this, &get_vtbl()};
}

static int allocate_segment_entry(void *impl_ptr, SrmMsgSegment *segment) noexcept;

static SrmStrView err_to_str(int err) noexcept;

static const SrmMsgBuilderVtbl& get_vtbl() noexcept {
    static const SrmMsgBuilderVtbl vtbl = {
        allocate_segment_entry,
        err_to_str
    };

    return vtbl;
}

static int allocate_segment_entry(void *impl_ptr, SrmMsgSegment *segment) noexcept {
    assert(impl_ptr);
    assert(segment);
    assert(segment->len > 0);

    auto &builder = *static_cast<MsgBuilder*>(impl_ptr);

    try {
        auto allocated = builder.allocateSegment(static_cast<capnp::uint>(segment->len));

        segment->data = reinterpret_cast<SrmWord*>(allocated.begin());
        segment->len = static_cast<SrmIndex>(allocated.end() - allocated.begin());

        return 0;
    } catch (const std::bad_alloc&) {
        return 1;
    } catch (...) {
        return 2;
    }
}

static SrmStrView make_str_view(std::string_view sv);

static SrmStrView err_to_str(int err) noexcept {
    switch (err) {
    case 0: return make_str_view("ok");
    case 1: return make_str_view("out of memory");
    case 2: return make_str_view("unknown error");
    }

    return {nullptr, 0};
}

static SrmStrView make_str_view(std::string_view sv) {
    return {sv.data(), static_cast<SrmIndex>(sv.size())};
}

} // namespace srm
