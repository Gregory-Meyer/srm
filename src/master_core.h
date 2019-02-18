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

#include "err.h"
#include <srm/core.h>

#include <atomic>
#include <string_view>
#include <thread>
#include <utility>
#include <vector>

#include <tbb/concurrent_hash_map.h>
#include <tbb/concurrent_vector.h>
#include <tbb/task_arena.h>

namespace srm {

/**
 *  MasterCore is an in-memory SRM core that statically loads nodes.
 *
 *  Nodes are loaded upon construction and their lifetime is until the
 *  core is killed.
 */
class MasterCore {
public:
    void subscribe(SrmSubscriberParams params);

    void publish(SrmPublishParams params);

    SrmCore as_core() noexcept;

private:
    void throw_if_shutting_down(std::string_view what) const;

    class Callback {
    public:
        constexpr Callback(SrmSubscribeCallback cb, void *arg) noexcept : fn_(cb), arg_(arg) { }

        int operator()(SrmCore core, SrmMsgView msg) const noexcept {
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
    tbb::task_arena arena_;
    std::atomic<bool> shutting_down_;
};

/**
 *  Thrown when an operation is launched while the core is shutting
 *  down.
 */
class CoreShuttingDown : public Error {
public:
    /**
     *  @param what Must be explicitly convertible to std::string.
     *
     *  @throws std::bad_alloc
     */
    template <typename S, std::enable_if_t<std::is_constructible_v<std::string, S>, int> = 0>
    explicit CoreShuttingDown(S &&what) : Error(std::forward<S>(what)) { }

    virtual ~CoreShuttingDown() = default;
};

} // namespace srm

#endif
