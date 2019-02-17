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

#ifndef SRM_IMPL_MASTER_CORE_H
#define SRM_IMPL_MASTER_CORE_H

#include <srm/core.h>

#include <string_view>
#include <utility>

#include <tbb/concurrent_hash_map.h>
#include <tbb/concurrent_vector.h>

namespace srm {

class MasterCore {
public:
    void subscribe(SrmSubscriberParams params);

    void publish(SrmPublishParams params);

    static const SrmCoreVtbl& vtbl() noexcept;

    static std::string_view err_to_str(int err) noexcept;

private:
    class Callback {
    public:
        constexpr Callback(SrmSubscribeCallback cb, void *arg) noexcept : fn_(cb), arg_(arg) { }

        int operator()(SrmCore *core, SrmMsgView msg) noexcept {
            return fn_(core, msg, arg_);
        }

    private:
        SrmSubscribeCallback fn_;
        void *arg_;
    };

    using SubscriptionKey = std::pair<std::string, SrmMsgType>;
    using CallbackVec = tbb::concurrent_vector<Callback>;
    using SubscriberTable = tbb::concurrent_hash_map<SubscriptionKey, CallbackVec>;

    SubscriberTable subscribers_;
};

} // namespace srm

#endif
