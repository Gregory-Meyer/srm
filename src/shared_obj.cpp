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

#include "shared_obj.h"

#include "plat.h"

#include <cassert>
#include <iostream>
#include <utility>

#ifdef SRM_WINDOWS
#include <windows.h>
#elif defined(SRM_POSIX)
#include <dlfcn.h>
#endif

namespace srm {

#ifdef SRM_WINDOWS

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
SharedObj::SharedObj(const char *filename) : obj_(LoadLibraryA(filename)) {
    assert(filename);

    if (obj_) {
        return;
    }

    throw LoadError("srm::SharedObj::SharedObj", get_last_error());
}

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
SharedObj::~SharedObj() try {
    if (!obj_) {
        return;
    }

    if (FreeLibrary(obj_)) {
        return;
    }

    throw UnloadError("srm::SharedObj::~SharedObj", get_last_error());
} catch (const UnloadError &e) {
    // you may ask yourself, why do this?
    // the answer is so that I can see the exception when debugging
    std::cerr << "srm::UnloadError: " << e.what() << ": " << e.nested().what() << std::endl;
}

void* SharedObj::resolve_impl(const char *symbol) const {
    assert(obj_);
    assert(symbol);

    void *const addr = GetProcAddress(obj_, symbol);

    if (addr) {
        return addr;
    }

    throw SymbolResolutionError("srm::SharedObj::resolve_impl", get_last_error());
}

#elif defined(SRM_POSIX)

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
SharedObj::SharedObj(const char *filename)
: obj_(dlopen(filename, RTLD_NOW | RTLD_LOCAL)) {
    assert(filename);

    if (obj_) {
        return;
    }

    throw LoadError("srm::SharedObj::SharedObj", std::runtime_error(dlerror()));
}

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
SharedObj::~SharedObj() try {
    if (!obj_) {
        return;
    }

    if (dlclose(obj_) == 0) {
        return;
    }

    throw UnloadError("srm::SharedObj::~SharedObj", std::runtime_error(dlerror()));
} catch (const UnloadError &e) {
    // you may ask yourself, why do this?
    // the answer is so that I can see the exception when debugging
    std::cerr << "srm::UnloadError: " << e.what() << ": " << e.nested().what() << std::endl;
}

void* SharedObj::resolve_impl(const char *symbol) const {
    assert(obj_);
    assert(symbol);

    void *const addr = dlsym(obj_, symbol);

    if (addr) {
        return addr;
    }

    throw SymbolResolutionError("srm::SharedObj::resolve_impl", std::runtime_error(dlerror()));
}

#endif

/** @param other After invocation, will be unusable. */
SharedObj::SharedObj(SharedObj &&other) noexcept : obj_(other.obj_) {
    other.obj_ = nullptr;
}

/** @param other Will be swapped with *this. */
SharedObj& SharedObj::operator=(SharedObj &&other) noexcept {
    std::swap(obj_, other.obj_);

    return *this;
}

} // namespace srm
