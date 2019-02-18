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

#include "fnv1a.h"

#include <climits>
#include <numeric>

constexpr std::size_t operator""_usize(unsigned long long x) noexcept {
    return static_cast<std::size_t>(x);
}

constexpr std::size_t get_prime() noexcept {
    static_assert(sizeof(std::size_t) * CHAR_BIT == 32 || sizeof(std::size_t) * CHAR_BIT == 64);

    if constexpr (sizeof(std::size_t) * CHAR_BIT == 32) {
        return 16777619_usize;
    } else {
        return 1099511628211_usize;
    }
}

constexpr std::size_t get_offset_basis() noexcept {
    static_assert(sizeof(std::size_t) * CHAR_BIT == 32 || sizeof(std::size_t) * CHAR_BIT == 64);

    if constexpr (sizeof(std::size_t) * CHAR_BIT == 32) {
        return 2166136261_usize;
    } else {
        return 14695981039346656037_usize;
    }
}

std::size_t fnv1a(const void *data, std::size_t len) noexcept {
    const auto as_bytes = static_cast<const unsigned char*>(data);

    return std::accumulate(
        as_bytes, as_bytes + len, get_offset_basis(),
        [](std::size_t hash, unsigned char byte) {
            hash ^= static_cast<std::size_t>(byte);
            hash *= get_prime();

            return hash;
        }
    );
}
