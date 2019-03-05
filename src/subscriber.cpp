#include <cassert>
#include <cstdint>
#include <algorithm>
#include <fstream>
#include <vector>

#include <capnp/message.h>
#include <srm/core.h>
#include <srm/node.h>

#include "../capnp/message.capnp.h"

constexpr std::uint64_t TYPE = 0x93c2012830d68d3cull;

class Subscriber {
public:
    explicit Subscriber(SrmCore core, SrmStrView name) noexcept : core_(core), name_(name) {
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

        const char *const data = reader.getMsg().begin();
        const auto len = static_cast<SrmIndex>(reader.getMsg().size());

        [[gnu::unused]] const int res =
            core_.vptr->log_info(core_.impl_ptr, SrmStrView{ data, len });
        assert(res == 0);
    }

    static int callback_entry(SrmMsgView msg, void *arg) noexcept {
        static_cast<Subscriber*>(arg)->callback(msg);

        return 0;
    }

private:
    SrmCore core_;
    SrmStrView name_;
    SrmSubscriber subscriber_;
};

extern "C" {

static int create(SrmCore core, SrmStrView name, void **impl) noexcept {
    *impl = new Subscriber(core, name);

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
