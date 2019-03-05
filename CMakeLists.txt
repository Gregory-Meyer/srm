cmake_minimum_required(VERSION 3.1)

project(srm)

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(CMAKE_C_STANDARD 90)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

find_package(CapnProto REQUIRED)

include_directories(include src)

add_library(srm-publisher SHARED src/publisher.cpp capnp/message.capnp.c++)
target_link_libraries(srm-publisher capnp)

add_library(srm-subscriber SHARED src/subscriber.cpp capnp/message.capnp.c++)
target_link_libraries(srm-subscriber capnp)

install(TARGETS srm-publisher srm-subscriber LIBRARY DESTINATION lib)
