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

#ifndef SRM_IMPL_FNV1A_H
#define SRM_IMPL_FNV1A_H

#include <cstddef>
#include <string>
#include <string_view>

namespace srm {

std::size_t fnv1a(const void *data, std::size_t len) noexcept;

template <typename T, std::enable_if_t<
    std::is_arithmetic_v<T> || std::is_pointer_v<T> || std::is_member_pointer_v<T>,
    int
> = 0>
std::size_t fnv1a(T t) noexcept {
    return fnv1a(&t, sizeof(T));
}

template <typename C, typename T>
std::size_t fnv1a(const std::basic_string<C, T> &str) noexcept {
    return fnv1a(str.data(), static_cast<std::size_t>(str.size()));
}

template <typename C, typename T>
std::size_t fnv1a(std::basic_string_view<C, T> sv) noexcept {
    return fnv1a(sv.data(), static_cast<std::size_t>(sv.size()));
}

} // namespace srm

#endif
