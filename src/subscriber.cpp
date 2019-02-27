#include <cassert>
#include <cstdint>
#include <algorithm>
#include <atomic>
#include <chrono>
#include <iostream>
#include <thread>
#include <vector>

#include <capnp/message.h>
#include <srm/core.h>
#include <srm/node.h>

#include "../capnp/message.capnp.h"

constexpr std::uint64_t TYPE = 0x93c2012830d68d3cull;

class Subscriber {
public:
    explicit Subscriber(SrmCore core) noexcept : core_(core) {
        SrmSubscribeParams params;
        params.msg_type = TYPE;
        params.topic = SrmStrView{ "foo", 3 };
        params.callback = &callback_entry;
        params.arg = this;

        [[gnu::unused]] const int res = core_.vptr->subscribe(core_.impl_ptr, params, &subscriber_);
        assert(res == 0);
    }

    ~Subscriber() {
        [[gnu::unused]] const int res = subscriber_.vptr->disconnect(subscriber_.impl_ptr);
        assert(res == 0);
    }

    void run() noexcept { }

    void stop() noexcept { }

    void callback(SrmMsgView msg) noexcept {
        std::vector<kj::ArrayPtr<const capnp::word>> segments;

        std::transform(
            msg.segments, msg.segments + msg.num_segments, std::back_inserter(segments),
            [](SrmMsgSegmentView segment) -> kj::ArrayPtr<const capnp::word> {
                return {reinterpret_cast<const capnp::word*>(segment.data),
                        static_cast<std::size_t>(segment.len)};
            }
        );

        capnp::SegmentArrayMessageReader segment_reader({segments.data(), segments.size()});
        Message::Reader reader = segment_reader.getRoot<Message>();

        std::cout.write(reader.getMsg().begin(),
                        static_cast<std::streamsize>(reader.getMsg().size())) << '\n';
    }

    static int callback_entry(SrmMsgView msg, void *arg) noexcept {
        static_cast<Subscriber*>(arg)->callback(msg);

        return 0;
    }

private:
    SrmCore core_;
    SrmSubscriber subscriber_;
    std::atomic<bool> keep_running_ = ATOMIC_VAR_INIT(0);
};

extern "C" {

static int create(SrmCore core, void **impl) noexcept {
    *impl = new Subscriber(core);

    return 0;
}

static int destroy(void *impl) noexcept {
    delete static_cast<Subscriber*>(impl);

    return 0;
}

static int run(void *impl) noexcept {
    static_cast<Subscriber*>(impl)->run();

    return 0;
}

static int stop(void *impl) noexcept {
    static_cast<Subscriber*>(impl)->stop();

    return 0;
}

static SrmStrView get_type(const void*) noexcept {
    return {"c++/subscriber", 14};
}

static SrmStrView get_err_msg(const void*, int) noexcept {
    return SrmStrView{nullptr, 0};
}

} // extern "C"

static const SrmNodeVtbl vtbl = {
    create,
    destroy,
    run,
    stop,
    get_type,
    get_err_msg
};

SRM_SHARED_OBJECT_EXPORT const SrmNodeVtbl* srm_Node_get_vtbl(void) {
    return &vtbl;
}
