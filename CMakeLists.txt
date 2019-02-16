cmake_minimum_required(VERSION 3.8)

project(srm)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

include_directories(include src external/tbb/include)

add_subdirectory(external/tbb)

set(SRM_SOURCES src/err.cpp src/immobile.cpp src/plat.cpp src/shared_obj.cpp)

add_executable(srm ${SRM_SOURCES})
target_link_libraries(srm tbb_static)
