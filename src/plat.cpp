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

#include "plat.h"

#ifdef SRM_WINDOWS
#include <windows.h>
#endif

namespace srm {

#ifdef SRM_WINDOWS

class WindowsCategory : public std::error_category {
public:
    virtual ~WindowsCategory() = default;

    const char* name() const noexcept override {
        return "srm::WindowsCategory";
    }

    std::string message(int condition) const override {
        const auto err = static_cast<DWORD>(condition);

        char *as_string = nullptr;

        constexpr DWORD FLAGS = FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS
                                | FORMAT_MESSAGE_ARGUMENT_ARRAY | FORMAT_MESSAGE_ALLOCATE_BUFFER;
        FormatMessage(FLAGS, nullptr, err, 0, reinterpret_cast<char*>(&as_string), 0, nullptr);

        return std::string(as_string);
    }
};

std::system_error get_last_error() noexcept {
    return std::system_error(static_cast<int>(GetLastError()), windows_category());
}

const std::error_category& windows_category() noexcept {
    static const WindowsCategory category;

    return category;
}

#elif defined(SRM_POSIX)

std::system_error get_last_error() noexcept {
    return std::system_error(errno, std::generic_category());
}

#endif

} // namespace srm
