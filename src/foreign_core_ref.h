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

#ifndef SRM_IMPL_FOREIGN_CORE_REF_H
#define SRM_IMPL_FOREIGN_CORE_REF_H

#include "core.h"

#include <system_error>

namespace srm {

class ForeignCoreRef : public Core {
public:
    Expected<void> publish(SrmPublishParams params) noexcept override {
        const int err = srm_Core_publish(core_, params);

        if (err == 0) {
            return { };
        }

        return std::error_code(err, category_);
    }

    Expected<void> subscribe(SrmSubscriberParams params) noexcept override {
        const int err = srm_Core_subscribe(core_, params);

        if (err == 0) {
            return { };
        }

        return std::error_code(err, category_);
    }

    const SrmCoreVtbl& vtbl() const noexcept override {
        return *core_.vtbl;
    }

private:
    class ForeignCoreCategory : public std::error_category {
    public:
        explicit ForeignCoreCategory(const SrmCoreVtbl &vtbl) noexcept
        : err_to_str_(vtbl.err_to_str) { }

        const char* name() const noexcept override {
            return "srm::ForeignCoreRef::ForeignCoreCategory";
        }

        std::string message(int err) const override {
            const SrmStrView str = err_to_str_(err);

            return std::string(str.data, static_cast<std::string::size_type>(str.len));
        }

    private:
        SrmStrView (*err_to_str_)(int err);
    };

    SrmCore core_;
    ForeignCoreCategory category_;
};

} // namespace srm

#endif
