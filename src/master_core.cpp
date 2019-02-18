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

#include "master_core.h"

#include "msg_builder.h"

#include <cassert>
#include <functional>
#include <memory>
#include <tuple>

#include <tbb/parallel_for.h>

namespace srm {

enum class MasterError {
    Ok,
    OutOfMemory,
    ShuttingDown,
    Unknown,
};

std::string as_string(SrmStrView view) {
    assert(view.data);
    assert(view.len >= 0);

    return std::string(view.data, static_cast<std::string::size_type>(view.len));
}

void MasterCore::subscribe(SrmSubscriberParams params) {
    throw_if_shutting_down("MasterCore::subscribe");

    const SubscriptionKey key(as_string(params.topic), params.type);

    SubscriberTable::accessor callbacks;
    subscribers_.insert(callbacks, key);

    throw_if_shutting_down("MasterCore::subscribe");

    callbacks->second.push_back(Callback(params.callback, params.arg));
}

static SrmMsgView as_msg_view(capnp::MessageBuilder &builder, SrmMsgType type) {
    auto segments = builder.getSegmentsForOutput();

    return {
        reinterpret_cast<const SrmMsgSegmentView*>(segments.begin()),
        static_cast<SrmIndex>(segments.end() - segments.begin()),
        type
    };
}

void MasterCore::publish(SrmPublishParams params) {
    throw_if_shutting_down("MasterCore::publish");

    auto builder_ptr = std::make_shared<MsgBuilder>();

    params.fn(as_core(), builder_ptr->as_builder(), params.arg);

    const SrmMsgView msg_view = as_msg_view(*builder_ptr, params.type);
    SubscriptionKey key(as_string(params.topic), params.type);

    arena_.enqueue([this, builder_ptr = std::move(builder_ptr), key = std::move(key), msg_view] {
        throw_if_shutting_down("MasterCore::publish");

        SubscriberTable::const_accessor callbacks;

        if (!subscribers_.find(callbacks, key)) {
            // no callbacks for this topic+msg type...
            return;
        }

        // execute each callback in parallel
        tbb::parallel_for(
            callbacks->second.range(),
            [this, msg_view](const Callback &cb) {
                throw_if_shutting_down("MasterCore::publish");

                cb(as_core(), msg_view);
            }
        );
    });
}

template <typename C, std::enable_if_t<std::is_invocable_v<C>, int> = 0>
int invoke(C &&callable) noexcept {
    try {
        std::invoke(std::forward<C>(callable));
    } catch (const std::bad_alloc&) {
        return static_cast<int>(MasterError::OutOfMemory);
    } catch (const CoreShuttingDown&) {
        return static_cast<int>(MasterError::ShuttingDown);
    } catch (...) {
        return static_cast<int>(MasterError::Unknown);
    }

    return static_cast<int>(MasterError::Ok);
}

static int subscribe_entry(void *impl_ptr, SrmSubscriberParams params) noexcept {
    return invoke([&] { static_cast<MasterCore*>(impl_ptr)->subscribe(params); });
}

static int publish_entry(void *impl_ptr, SrmPublishParams params) noexcept {
    return invoke([&] { static_cast<MasterCore*>(impl_ptr)->publish(params); });
}

static SrmStrView err_to_str(int err) noexcept {
    auto str = [err]() -> std::string_view {
        switch (static_cast<MasterError>(err)) {
        case MasterError::Ok: return "ok";
        case MasterError::OutOfMemory: return "out of memory";
        case MasterError::ShuttingDown: return "shutting down";
        case MasterError::Unknown: return "unknown";
        }

        return std::string_view();
    }();

    return { str.data(), static_cast<SrmIndex>(str.size()) };
}

static const SrmCoreVtbl& get_vtbl() noexcept {
    static const SrmCoreVtbl vtbl = {
        &subscribe_entry,
        &publish_entry,
        &err_to_str,
    };

    return vtbl;
};

SrmCore MasterCore::as_core() noexcept {
    return {this, &get_vtbl()};
}

void MasterCore::throw_if_shutting_down(std::string_view what) const {
    if (shutting_down_.load()) {
        throw CoreShuttingDown(what);
    }
}

} // namespace srm
