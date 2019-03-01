#include <cassert>
#include <cstdint>
#include <atomic>
#include <chrono>
#include <thread>

#include <capnp/message.h>
#include <srm/core.h>
#include <srm/node.h>

#include "../capnp/message.capnp.h"

constexpr std::uint64_t TYPE = 0x93c2012830d68d3cull;

class RemoteBuilder : public capnp::MessageBuilder {
public:
    explicit RemoteBuilder(SrmMsgBuilder builder) noexcept : builder_(builder) { }

    kj::ArrayPtr<capnp::word> allocateSegment(capnp::uint minimum_size) override {
        SrmMsgSegment segment{
            nullptr,
            static_cast<std::ptrdiff_t>(minimum_size)
        };

        [[gnu::unused]] const int res = builder_.vptr->alloc_segment(builder_.impl_ptr, &segment);
        assert(res == 0);

        return {reinterpret_cast<capnp::word*>(segment.data),
                static_cast<std::size_t>(segment.len)};
    }

private:
    SrmMsgBuilder builder_;
};

class Publisher {
public:
    explicit Publisher(SrmCore core, SrmStrView name) noexcept : core_(core), name_(name) {
        SrmAdvertiseParams params;
        params.msg_type = TYPE;
        params.topic = SrmStrView{"foo", 3};

        [[gnu::unused]] const int res = core_.vptr->advertise(core_.impl_ptr, params, &publisher_);
        assert(res == 0);
    }

    ~Publisher() {
        [[gnu::unused]] const int res = publisher_.vptr->disconnect(publisher_.impl_ptr);
        assert(res == 0);
    }

    void run() noexcept {
        while (keep_running_.load()) {
            [[gnu::unused]] const int res =
                publisher_.vptr->publish(publisher_.impl_ptr, do_publish_entry, this);
            assert(res == 0);

            // std::this_thread::sleep_for(std::chrono::seconds(1));
        }
    }

    void stop() noexcept {
        keep_running_.store(false);
    }

    void do_publish(SrmMsgBuilder raw_builder) noexcept {
        RemoteBuilder builder(raw_builder);

        Message::Builder chatter = builder.initRoot<Message>();
        chatter.setMsg("Hello, world!");
    }

    static int do_publish_entry(SrmMsgBuilder builder, void *arg) noexcept {
        static_cast<Publisher*>(arg)->do_publish(builder);

        return 0;
    }

private:
    SrmCore core_;
    SrmStrView name_;
    SrmPublisher publisher_;
    std::atomic<bool> keep_running_ = ATOMIC_VAR_INIT(true);
};

extern "C" {

static int create(SrmCore core, SrmStrView name, void **impl) noexcept {
    *impl = new Publisher(core, name);

    return 0;
}

static int destroy(void *impl) noexcept {
    delete static_cast<Publisher*>(impl);

    return 0;
}

static int run(void *impl) noexcept {
    static_cast<Publisher*>(impl)->run();

    return 0;
}

static int stop(void *impl) noexcept {
    static_cast<Publisher*>(impl)->stop();

    return 0;
}

static SrmStrView get_type(const void*) noexcept {
    return {"c++/publisher", 13};
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
