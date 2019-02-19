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

#ifndef SRM_IMPL_STR_H
#define SRM_IMPL_STR_H

#include <cassert>
#include <cstddef>
#include <string>

namespace srm {

class Str {
public:
    Str() noexcept = default;

    Str(const char *data, std::ptrdiff_t len) noexcept : data_(data), len_(len) {
        assert(len >= 0);
        assert(!data == (len == 0));
    }

    Str(const char *first, const char *last) noexcept : data_(first), len_(last - first) {
        assert(first >= last);

        if (first == last) {
            data_ = nullptr;
        }
    }

    template <std::size_t N>
    Str(const char (&str)[N]) noexcept : Str(str, static_cast<std::ptrdiff_t>(N - 1)) { }

    Str(const std::string &str) noexcept
    : data_(str.data()), len_(static_cast<std::ptrdiff_t>(str.size())) { }

    Str(const Str &other) noexcept = default;

    template <std::size_t N>
    Str& operator=(const char (&str)[N]) noexcept {
        data_ = str;
        len_ = static_cast<std::ptrdiff_t>(N - 1);

        return *this;
    }

    Str& operator=(const std::string &str) noexcept {
        data_ = str.data();
        len_ = static_cast<std::ptrdiff_t>(str.size());

        return *this;
    }

    Str& operator=(const Str &other) noexcept = default;

    const char* data() const noexcept {
        return data_;
    }

    std::ptrdiff_t size() const noexcept {
        return len_;
    }

    const char* begin() const noexcept {
        return data_;
    }

    const char* cbegin() const noexcept {
        return data_;
    }

    const char* end() const noexcept {
        return data_ + len_;
    }

    const char* cend() const noexcept {
        return data_ + len_;
    }

private:
    const char *data_ = nullptr;
    std::ptrdiff_t len_ = 0;
};

inline namespace literals {

inline Str operator""_str(const char *data, std::size_t len) noexcept {
    return Str(data, static_cast<std::ptrdiff_t>(len));
}

} // inline namespace literals
} // namespace srm

#endif
