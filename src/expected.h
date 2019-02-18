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

#ifndef SRM_IMPL_EXPECTED_H
#define SRM_IMPL_EXPECTED_H

#include "macro.h"

#include <functional>
#include <initializer_list>
#include <system_error>
#include <variant>

namespace srm {

/**
 *  Expected is an exceptionless error handling primitive.
 *
 *  Expected is a tagged union of T and std::error_code. All operations
 *  that could fail on an Expected, such as dereferencing a value when
 *  the Expected holds an error, will instead print a message to stderr
 *  and call std::abort. It is required that Expected is constructed
 *  through methods that will not throw exceptions, as Expected should
 *  be usable in exceptionless environments.
 *
 *  T must not be constructible from a std::error_code.
 */
template <typename T>
class Expected {
private:
    template <typename ...Ts>
    static inline constexpr bool IS_IMPLICITLY_CONSTRUCTIBLE =
        std::is_nothrow_constructible_v<T, Ts...>
        && (sizeof...(Ts) != 1 || std::is_convertible_v<Ts..., T>)
        && (sizeof...(Ts) == 0 || !std::is_constructible_v<std::error_code, Ts...>);

    template <typename ...Ts>
    static inline constexpr bool IS_EXPLICITLY_CONSTRUCTIBLE =
        std::is_nothrow_constructible_v<T, Ts...>
        && (sizeof...(Ts) == 1 && !std::is_convertible_v<Ts..., T>)
        && !std::is_constructible_v<std::error_code, Ts...>;

public:
    static_assert(!std::is_constructible_v<T, std::error_code&>);
    static_assert(!std::is_constructible_v<T, const std::error_code&>);
    static_assert(!std::is_constructible_v<T, std::error_code&&>);
    static_assert(!std::is_constructible_v<T, const std::error_code&&>);

    /**
     *  Implicitly construct an Expected from a variadic number of
     *  arguments.
     *
     *  T must be nothrow constructible from (Ts...). If there is
     *  one argument U in (Ts...), U must be implicitly convertible to
     *  T.
     *
     *  @param ts... Perfect forwarded to the constructor of T.
     */
    template <typename ...Ts, std::enable_if_t<IS_IMPLICITLY_CONSTRUCTIBLE<Ts...>, int> = 0>
    constexpr Expected(Ts &&...ts) noexcept : state_(std::forward<Ts>(ts)...) { }

    /**
     *  Explicitly construct an Expected from a variadic number of
     *  arguments.
     *
     *  T must be nothrow constructible from U and U must not be
     *  implicitly convertible to T.
     *
     *  @param u Perfect forwarded to the constructor of T.
     */
    template <typename U, std::enable_if_t<IS_EXPLICITLY_CONSTRUCTIBLE<U>, int> = 0>
    constexpr explicit Expected(U &&u) noexcept : state_(std::forward<U>(u)) { }

    /**
     *  Implicitly construct an Expected from an initializer list
     *  and a variadic number of arguments.
     *
     *  T must be nothrow constructible from
     *  (std::initializer_list<U>&, Ts...). If there are no arguments
     *  in (ts...), std::initializer_list<U>& must be implicitly
     *  convertible to T.
     *
     *  @param list Forwarded to the constructor of T.
     *  @param ts... Perfect forwarded to the constructor of T.
     */
    template <typename U, typename ...Ts, std::enable_if_t<
        IS_IMPLICITLY_CONSTRUCTIBLE<std::initializer_list<U>&, Ts...>,
        int
    > = 0>
    constexpr Expected(std::initializer_list<U> list, Ts &&...ts) noexcept
    : state_(list, std::forward<Ts>(ts)...) { }

    /**
     *  Explicitly construct an Expected from an initializer list.
     *
     *  T must be nothrow constructible from std::initializer_list<U>&.
     *  std::initializer_list<U>& must not be implicitly convertible to
     *  T.
     *
     *  @param list Forwarded to the constructor of T.
     */
    template <typename U, std::enable_if_t<
        IS_EXPLICITLY_CONSTRUCTIBLE<std::initializer_list<U>&>,
        int
    > = 0>
    constexpr explicit Expected(std::initializer_list<U> list) noexcept : state_(list) { }

    /** Construct an Expected from a std::error_code. */
    Expected(std::error_code code) noexcept : state_(code) { }

    /** @returns True if this Expected holds T. */
    constexpr bool has_value() const noexcept {
        return std::holds_alternative<T>(state_);
    }

    /** @returns True if this Expected holds std::error_code. */
    constexpr bool has_error() const noexcept {
        return std::holds_alternative<std::error_code>(state_);
    }

    /**
     *  If this Expected does not hold T, an error message will
     *  be printed to stderr and std::abort() will be invoked.
     */
    constexpr T& value() noexcept {
        T *const maybe_value = std::get_if<T>(&state_);
        SRM_EXPECT(maybe_value, "Expected holds an error");

        return *maybe_value;
    }

    /**
     *  If this Expected does not hold T, an error message will
     *  be printed to stderr and std::abort() will be invoked.
     */
    constexpr const T& value() const noexcept {
        const T *const maybe_value = std::get_if<T>(&state_);
        SRM_EXPECT(maybe_value, "Expected holds an error");

        return *maybe_value;
    }

    /**
     *  If this Expected does not hold T, an error message will
     *  be printed to stderr and std::abort() will be invoked.
     */
    constexpr T& operator*() noexcept {
        T *const maybe_value = std::get_if<T>(&state_);
        SRM_EXPECT(maybe_value, "Expected holds an error");

        return *maybe_value;
    }

    /**
     *  If this Expected does not hold T, an error message will
     *  be printed to stderr and std::abort() will be invoked.
     */
    constexpr const T& operator*() const noexcept {
        const T *const maybe_value = std::get_if<T>(&state_);
        SRM_EXPECT(maybe_value, "Expected holds an error");

        return *maybe_value;
    }

    /**
     *  If this Expected does not hold T, an error message will
     *  be printed to stderr and std::abort() will be invoked.
     */
    constexpr T* operator->() noexcept {
        T *const maybe_value = std::get_if<T>(&state_);
        SRM_EXPECT(maybe_value, "Expected holds an error");

        return maybe_value;
    }

    /**
     *  If this Expected does not hold T, an error message will
     *  be printed to stderr and std::abort() will be invoked.
     */
    constexpr const T* operator->() const noexcept {
        const T *const maybe_value = std::get_if<T>(&state_);
        SRM_EXPECT(maybe_value, "Expected holds an error");

        return maybe_value;
    }

    /**
     *  If this Expected does not hold std::error_code, an error
     *  message will be printed to stderr and std::abort() will be
     *  invoked.
     */
    std::error_code error() const noexcept {
        const std::error_code *const maybe_error = std::get_if<std::error_code>(&state_);
        SRM_EXPECT(maybe_error, "Expected holds a value");

        return *maybe_error;
    }

private:
    std::variant<T, std::error_code> state_;
};

template <>
class Expected<void> {
public:
    constexpr Expected() noexcept : state_(std::monostate()) { }

    Expected(std::error_code code) noexcept : state_(code) { }

    constexpr bool has_value() const noexcept {
        return std::holds_alternative<std::monostate>(state_);
    }

    constexpr bool has_error() const noexcept {
        return std::holds_alternative<std::error_code>(state_);
    }

    std::error_code error() const noexcept {
        const std::error_code *const maybe_error = std::get_if<std::error_code>(&state_);
        SRM_EXPECT(maybe_error, "Expected holds a value");

        return *maybe_error;
    }

private:
    std::variant<std::monostate, std::error_code> state_;
};

} // namespace srm

#endif
