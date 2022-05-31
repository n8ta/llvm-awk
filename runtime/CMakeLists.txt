cmake_minimum_required(VERSION 3.22)
project(llvm-awk-runtime)

set(CMAKE_CXX_STANDARD 17)

add_library(runtime runtime.cpp)
