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

#ifndef SRM_IMPL_SHARED_OBJ_H
#define SRM_IMPL_SHARED_OBJ_H

#include "err.h"
#include "immobile.h"

#include <memory>
#include <system_error>

namespace srm {

/** SharedObj manages a dynamically loaded object. */
class SharedObj : Immobile {
public:
    /**
     *  Opens a shared object.
     *
     *  Implemented using dlopen() on POSIX-compliant systems and
     *  LoadLibraryA() on Windows.
     *
     *  @param filename Must not be NULL and must be null-terminated.
     *
     *  @throws LoadError If no shared object could be loaded.
     */
    explicit SharedObj(const char *filename);

    /**
     *  Closes the owned shared object.
     *
     *  Implemented using dlclose() on POSIX-compliant systems and
     *  FreeLibrary() on Windows.
     *
     *  @throws UnloadError If the managed shared object could not be
     *                      unloaded. However, the exception will be
     *                      caught and printed using std::cerr if
     *                      thrown.
     */
    ~SharedObj();

    /**
     *  Resolves a symbol from the owned shared object.
     *
     *  Implemented using dlsym() on POSIX-compliant systems and
     *  GetProcAddress() on Windows.
     *
     *  @param symbol Must not be NULL and must be null-terminated.
     *
     *  @throws SymbolResolutionError If the requested symbol could not
     *                                be resolved.
     */
    template <typename S>
    S& resolve(const char *symbol) const {
        return *reinterpret_cast<S*>(resolve_impl(symbol));
    }

private:
    void* resolve_impl(const char *symbol) const;

    void *obj_;
};

/**
 *  Base class for errors originating from a SharedObj.
 *
 *  Owns an atomically refcounted exception.
 */
class SharedObjError : public Error {
public:
    /**
     *  @param what Must be explicitly convertible to std::string.
     *  @param nested Must have std::exception as a public base class.
     *
     *  @throws std::bad_alloc
     */
    template <typename S, typename E, std::enable_if_t<
        std::is_constructible_v<std::string, S>
        && std::is_base_of_v<std::exception, std::decay_t<E>>,
        int
    > = 0>
    SharedObjError(S &&what, E &&nested)
    : Error(std::make_shared<std::string>(std::forward<S>(what))),
      nested_(std::make_shared<std::decay_t<E>>(std::forward<E>(nested))) { }

    virtual ~SharedObjError() = default;

    const std::exception& nested() const noexcept {
        return *nested_;
    }

private:
    std::shared_ptr<std::exception> nested_;
};

/** Thrown if SharedObj fails to load a shared object. */
class LoadError : public SharedObjError {
public:
    /**
     *  @param what Must be explicitly convertible to std::string.
     *  @param nested Must have std::exception as a public base class.
     *
     *  @throws std::bad_alloc
     */
    template <typename S, typename E, std::enable_if_t<
        std::is_constructible_v<std::string, S>
        && std::is_base_of_v<std::exception, std::decay_t<E>>,
        int
    > = 0>
    LoadError(S &&what, E &&nested)
    : SharedObjError(std::forward<S>(what), std::forward<E>(nested)) { }

    virtual ~LoadError() = default;
};

/** Thrown if SharedObj fails to unload a shared object. */
class UnloadError : public SharedObjError {
public:
    /**
     *  @param what Must be explicitly convertible to std::string.
     *  @param nested Must have std::exception as a public base class.
     *
     *  @throws std::bad_alloc
     */
    template <typename S, typename E, std::enable_if_t<
        std::is_constructible_v<std::string, S>
        && std::is_base_of_v<std::exception, std::decay_t<E>>,
        int
    > = 0>
    UnloadError(S &&what, E &&nested)
    : SharedObjError(std::forward<S>(what), std::forward<E>(nested)) { }

    virtual ~UnloadError() = default;
};

/** Thrown if SharedObj fails to resolve a symbol in a shared object. */
class SymbolResolutionError : public SharedObjError {
public:
    /**
     *  @param what Must be explicitly convertible to std::string.
     *  @param nested Must have std::exception as a public base class.
     *
     *  @throws std::bad_alloc
     */
    template <typename S, typename E, std::enable_if_t<
        std::is_constructible_v<std::string, S>
        && std::is_base_of_v<std::exception, std::decay_t<E>>,
        int
    > = 0>
    SymbolResolutionError(S &&what, E &&nested)
    : SharedObjError(std::forward<S>(what), std::forward<E>(nested)) { }

    virtual ~SymbolResolutionError() = default;
};

} // namespace srm

#endif
