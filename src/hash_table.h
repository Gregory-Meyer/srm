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

#ifndef SRM_IMPL_HASH_TABLE_H
#define SRM_IMPL_HASH_TABLE_H

#include "fnv1a.h"
#include "macro.h"

#include <memory>
#include <optional>
#include <utility>
#include <vector>

namespace srm {

template <typename T, typename A = std::allocator<T>>
class HashTable {
public:
    static_assert(std::is_nothrow_move_constructible_v<T>);
    static_assert(std::is_nothrow_move_assignable_v<T>);
    static_assert(std::is_nothrow_swappable_v<T>);

    using value_type = std::pair<std::string, T>;

    std::pair<T&, bool> insert(std::string_view key, const T &value) {
        return emplace(key, value);
    }

    std::pair<T&, bool> insert(std::string_view key, T &&value) {
        return emplace(key, std::move(value));
    }

    std::pair<T&, bool> insert(std::string &&key, const T &value)
    noexcept(std::is_nothrow_copy_constructible_v<T>) {
        return emplace(std::move(key), value);
    }

    std::pair<T&, bool> insert(std::string &&key, T &&value) noexcept {
        return emplace(std::move(key), std::move(value));
    }

    T* find(std::string_view key) noexcept {
        const std::size_t hash = fnv1a(key);

        return find(key, hash);
    }

    template <typename ...Ts, std::enable_if_t<std::is_constructible_v<T, Ts...>, int> = 0>
    std::pair<T&, bool> emplace(std::string_view key, Ts &&...ts) {
        const std::size_t hash = fnv1a(key);

        T *const maybe_found = find(key, hash);

        if (maybe_found) {
            return {*maybe_found, false};
        }

        return {emplace_nonmember(std::string(key), hash, std::forward<Ts>(ts)...)};
    }

    template <typename ...Ts, std::enable_if_t<std::is_constructible_v<T, Ts...>, int> = 0>
    std::pair<T&, bool> emplace(std::string &&key, Ts &&...ts)
    noexcept(std::is_nothrow_constructible_v<T, Ts...>) {
        const std::size_t hash = fnv1a(key);

        T *const maybe_found = find(key, hash);

        if (maybe_found) {
            return {*maybe_found, false};
        }

        return {emplace_nonmember(std::move(key), hash, std::forward<Ts>(ts)...)};
    }

private:
    class Bucket {
    public:
        Bucket() noexcept = default;

        ~Bucket() {
            if (dib_ == EMPTY) {
                return;
            }

            reinterpret_cast<value_type&>(storage_).value_type::~value_type();
        }

        template <typename ...Ts, std::enable_if_t<
            std::is_constructible_v<value_type, Ts...>,
            int
        > = 0>
        std::pair<value_type&, bool> emplace_if_empty(unsigned char dib, Ts &&...ts)
        noexcept(std::is_nothrow_constructible_v<value_type, Ts...>) {
            if (dib_ != EMPTY) {
                return {reinterpret_cast<value_type&>(storage_), false};
            }

            ::new(static_cast<void*>(storage_)) value_type(std::forward<Ts>(ts)...);
            dib_ = dib;

            return {reinterpret_cast<value_type&>(storage_), true};
        }

        template <typename ...Ts, std::enable_if_t<
            std::is_constructible_v<value_type, Ts...>,
            int
        > = 0>
        std::tuple<value_type&, bool, std::optional<std::pair<unsigned char, value_type>>>
        emplace_if_empty_or_swap_if_less(unsigned char dib, Ts &&...ts)
        noexcept(std::is_nothrow_constructible_v<value_type, Ts...>) {
            std::tuple<value_type&, bool, std::optional<std::pair<unsigned char, value_type>>> ret(
                reinterpret_cast<value_type&>(storage_),
                false,
                std::nullopt
            );

            if (dib_ != EMPTY && dib_ >= dib) {
                return ret;
            }

            std::get<1>(ret) = true;

            if (dib_ != EMPTY) {
                std::get<2>(ret).emplace(dib_, std::move(reinterpret_cast<value_type&>(storage_)));
            }

            if constexpr (std::is_nothrow_constructible_v<value_type, Ts...>) {
                ::new(static_cast<void*>(storage_)) value_type(std::forward<Ts>(ts)...);
            } else {
                try {
                    ::new(static_cast<void*>(storage_)) value_type(std::forward<Ts>(ts)...);
                } catch (...) {
                    if (dib_ != EMPTY) {
                        auto &swapped = *std::get<2>(ret).second;
                        ::new(static_cast<void*>(storage_)) value_type(std::move(swapped));
                    }

                    throw;
                }
            }

            dib_ = dib;

            return ret;
        }

        T* value() noexcept {
            if (dib_ == EMPTY) {
                return nullptr;
            }

            return reinterpret_cast<value_type&>(storage_);
        }

        const T* value() const noexcept {
            if (dib_ == EMPTY) {
                return nullptr;
            }

            return reinterpret_cast<const value_type&>(storage_);
        }

        bool clear() noexcept {
            if (dib_ == EMPTY) {
                return false;
            }

            reinterpret_cast<value_type*>(storage_).value_type::~value_type();
            dib_ = EMPTY;
        }

    private:
        static inline constexpr unsigned char EMPTY = std::numeric_limits<unsigned char>::max();

        std::aligned_storage_t<sizeof(value_type), alignof(value_type)> storage_;
        unsigned char dib_ = EMPTY;
    };

    std::size_t modulo_index(std::size_t x) const noexcept {
        return x & (buckets_.size() - 1);
    }

    void grow() {
        HashTable grown;
        grown.buckets_.resize(buckets_.size() * 2);

        for (auto &bucket : buckets_) {
            value_type *maybe_value = bucket.value();

            if (maybe_value) {
                grown.insert(std::move(maybe_value->first), std::move(maybe_value->second));
                bucket.clear();
            }
        }

        *this = grown;
    }

    T* find(std::string_view key, std::size_t hash) noexcept {
        const std::size_t starting_index = modulo_index(hash);

        for (std::size_t i = starting_index; i < buckets_.size(); ++i) {
            value_type *maybe_value = buckets_[i].value();

            if (maybe_value && maybe_value->first == key) {
                return std::addressof(maybe_value->second);
            } else if (!maybe_value) {
                return nullptr;
            }
        }

        for (std::size_t i = 0; i < starting_index; ++i) {
            value_type *maybe_value = buckets_[i].value();

            if (maybe_value && maybe_value->first == key) {
                return std::addressof(maybe_value->second);
            } else if (!maybe_value) {
                return nullptr;
            }
        }

        return nullptr;
    }

    template <typename ...Ts, std::enable_if_t<std::is_constructible_v<T, Ts...>, int> = 0>
    T& emplace_nonmember(std::string &&key, std::size_t hash, Ts &&...ts)
    noexcept(std::is_nothrow_constructible_v<T, Ts...>) {
        if (size_ >= buckets_.size() / 2) {
            grow();
        }

        unsigned char dib = 0;
        const std::size_t starting_index = modulo_index(hash);

        for (std::size_t i = 0; i < buckets_.size(); ++i, ++dib) {
            auto result = buckets_[modulo_index(starting_index + i)]
                .emplace_if_empty_or_swap_if_less(dib, std::move(key), std::forward<Ts>(ts)...);

            if (std::get<2>(result)) {
                auto [swapped_dib, swapped] = *std::get<2>(result);

                for (std::size_t j = i + 1; j < buckets_.size(); ++i, ++swapped_dib) {
                    auto &bucket = buckets_[modulo_index(starting_index + j)];

                    if (bucket.emplace_if_empty(swapped_dib, std::move(swapped)).second) {
                        ++size_;

                        return {std::get<0>(result), true};
                    }
                }

                __builtin_unreachable();
            } else if (std::get<1>(result)) {
                ++size_;

                return {std::get<0>(result), true};
            }
        }

        __builtin_unreachable();
    }

    using Alloc = typename std::allocator_traits<A>::template rebind_alloc<Bucket>;

    std::vector<Bucket, Alloc> buckets_ = std::vector<Bucket, Alloc>(8);
    std::size_t size_ = 0;
};

} // namespace srm

#endif
