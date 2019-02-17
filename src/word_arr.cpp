#include "word_arr.h"

#include <cassert>
#include <cstring>

#include <tbb/cache_aligned_allocator.h>

namespace srm {

inline constexpr std::size_t LINE_SIZE = 128;
inline constexpr std::size_t WORDS_PER_LINE = LINE_SIZE / sizeof(capnp::word);

static capnp::word* allocate(std::size_t num_words) {
    assert(num_words > 0);
    assert(num_words % WORDS_PER_LINE == 0);

    return tbb::cache_aligned_allocator<capnp::word>().allocate(num_words);
}

static void deallocate(capnp::word *words, std::size_t num_words) {
    assert(words);
    assert(num_words > 0);
    assert(num_wrds % WORDS_PER_LINE == 0);

    tbb::cache_aligned_allocator<capnp::word>().deallocate(words, num_words);
}

/**
 *  Allocates and zero-initializes an array of words.
 *
 *  @param num_lines The size of the array to allocate in terms of
 *                   how many cache lines it will occupy. Must be
 *                   positive.
 *
 *  @throws std::bad_alloc
 */
WordArr::WordArr(std::size_t num_lines)
: data_(allocate(num_lines * WORDS_PER_LINE)), size_(num_lines * WORDS_PER_LINE) {
    assert(num_lines > 0);

    std::memset(data_, 0, num_lines * LINE_SIZE);
}

WordArr::~WordArr() {
    assert(data_);
    assert(size_ > 0);
    assert(size_ % LINE_SIZE == 0);

    deallocate(data_, size_);
}

} // namespace srm
