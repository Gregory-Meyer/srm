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

#ifndef SRM_IMPL_WORD_ARR_H
#define SRM_IMPL_WORD_ARR_H

#include <cstddef>
#include <utility>

#include <capnp/common.h>

namespace srm {

/**
 *  WordArr manages a cache-aligned buffer of memory for use in Cap'n
 *  Proto messages.
 *
 *  WordArr is backed by tbb::cache_aligned_allocator, which at the
 *  time of writing aligns types on 128-byte boundaries. Because of
 *  this, WordArr allocates space in blocks that are multiples of 128
 *  bytes.
 */
class WordArr {
public:
    /**
     *  Allocates and zero-initializes an array of words.
     *
     *  @param num_lines The size of the array to allocate in terms of
     *                   how many cache lines it will occupy. Must be
     *                   positive.
     *
     *  @throws std::bad_alloc
     */
    explicit WordArr(std::size_t num_lines);

    /** Creates a WordArr by moving resources from another WordArr. */
    constexpr WordArr(WordArr &&other) noexcept : data_(other.data_), size_(other.size_) {
        other.data_ = nullptr;
        other.size_ = 0;
    }

    /** Deallocates any memory owned by this WordArr. */
    ~WordArr();

    /** Exchanges this WordArr's resources with another. */
    WordArr& operator=(WordArr &&other) noexcept {
        std::swap(data_, other.data_);
        std::swap(size_, other.size_);

        return *this;
    }

    /** @returns A mutable pointer to this WordArr's array. */
    constexpr capnp::word* data() noexcept {
        return data_;
    }

    /** @returns A pointer to this WordArr's array. */
    constexpr const capnp::word* data() const noexcept {
        return data_;
    }

    /** @returns The size of this WordArr's array in words. */
    constexpr std::size_t size() const noexcept {
        return size_;
    }

private:
    capnp::word *data_;
    std::size_t size_;
};

} // namespace srm

#endif
