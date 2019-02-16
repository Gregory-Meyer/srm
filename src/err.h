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

#ifndef SRM_IMPL_ERR_H
#define SRM_IMPL_ERR_H

#include <cassert>
#include <exception>
#include <memory>
#include <string>

namespace srm {

/**
 *  Base class for errors in the SRM C++ backend.
 *
 *  Encapsulates an atomically refcounted null-terminated string.
 */
class Error : public std::exception {
public:
    /**
     *  Creates an Error with string allocated from the free store.
     *
     *  @param what The reason for this error. Typically the namespaced
     *              function name that the error is being thrown from.
     *              Perfect forwarded to the constructor of
     *              std::string, so it must be explicitly convertible
     *              to std::string.
     *
     *  @throws std::bad_alloc
     */
    template <typename S, std::enable_if_t<std::is_constructible_v<std::string, S>, int> = 0>
    explicit Error(S &&what)
    : what_(std::make_shared<const std::string>(std::forward<S>(what))) { }

    virtual ~Error() = default;

    /**
     *  @returns The argument provided to this Error upon construction.
     */
    const char* what() const noexcept override {
        assert(what_);

        return what_->c_str();
    }

private:
    std::shared_ptr<const std::string> what_;
};

} // namespace srm

#endif
