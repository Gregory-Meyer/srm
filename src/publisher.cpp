#include <cassert>
#include <cstdarg>
#include <cstdint>
#include <cstdio>
#include <atomic>
#include <chrono>
#include <thread>

#include <capnp/message.h>
#include <srm/core.h>
#include <srm/node.h>

#include "../capnp/message.capnp.h"

constexpr std::uint64_t TYPE = 0x93c2012830d68d3cull;

namespace {

[[gnu::format(printf, 1, 2)]] std::string format(const char *format, ...);

SrmStrView as_view(const std::string &s) noexcept;

SrmStrView operator""_sv(const char *data, std::size_t len) noexcept;

} // namespace

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
        params.topic = "foo"_sv;

        [[gnu::unused]] const int res = core_.vptr->advertise(core_.impl_ptr, params, &publisher_);
        assert(res == 0);

        const auto name_len = static_cast<int>(name_.len);
        param_name_ = format(".%*.*s.param", name_len, name_len, name_.data);
        [[gnu::unused]] const int param_res =
            core_.vptr->param_seti(core_.impl_ptr, as_view(param_name_), 0);
        assert(param_res == 0);
    }

    ~Publisher() {
        [[gnu::unused]] const int res = publisher_.vptr->disconnect(publisher_.impl_ptr);
        assert(res == 0);
    }

    void run() noexcept {
        while (keep_running_.load()) {
            [[gnu::unused]] const int pub_res =
                publisher_.vptr->publish(publisher_.impl_ptr, do_publish_entry, this);
            assert(pub_res == 0);

            std::ptrdiff_t value;
            [[gnu::unused]] const int param_res =
                core_.vptr->param_geti(core_.impl_ptr, as_view(param_name_), &value);
            assert(param_res == 0);

            core_.vptr->log_info(core_.impl_ptr, as_view(format("%s = %td", param_name_.data(), value)));
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
    std::string param_name_;
};

namespace {

[[gnu::format(printf, 1, 2)]] std::string format(const char *format, ...) {
    va_list args;
    va_start(args, format);

    va_list args_copy;
    va_copy(args_copy, args);

    const int bufsz = std::vsnprintf(nullptr, 0, format, args_copy);
    va_end(args_copy);

    assert(bufsz >= 0);
    std::string buf;

    try {
        buf.resize(static_cast<std::string::size_type>(bufsz));
    } catch (...) {
        va_end(args);

        throw;
    }

    [[gnu::unused]] const int ret =
        std::vsnprintf(&buf.front(), static_cast<std::size_t>(buf.size()) + 1, format, args);
    va_end(args);
    assert(ret >= 0);

    return buf;
}

SrmStrView as_view(const std::string &s) noexcept {
    return SrmStrView{s.data(), static_cast<SrmIndex>(s.size())};
}

SrmStrView operator""_sv(const char *data, std::size_t len) noexcept {
    return SrmStrView{data, static_cast<SrmIndex>(len)};
}

int create(SrmCore core, SrmStrView name, void **impl) noexcept {
    *impl = new Publisher(core, name);

    return 0;
}

int destroy(void *impl) noexcept {
    delete static_cast<Publisher*>(impl);

    return 0;
}

int run(void *impl) noexcept {
    static_cast<Publisher*>(impl)->run();

    return 0;
}

int stop(void *impl) noexcept {
    static_cast<Publisher*>(impl)->stop();

    return 0;
}

SrmStrView get_type(const void*) noexcept {
    return {"c++/publisher", 13};
}

SrmStrView get_err_msg(const void*, int) noexcept {
    return SrmStrView{nullptr, 0};
}

} // namespace

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
