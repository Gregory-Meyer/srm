cmake_minimum_required(VERSION 3.1)

project(srm)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(CMAKE_C_STANDARD 90)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

include_directories(include src)

add_library(srm-publisher STATIC src/publisher.cpp)
add_library(srm-subscriber STATIC src/subscriber.cpp)
