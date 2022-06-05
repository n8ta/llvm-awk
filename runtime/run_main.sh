#!/bin/sh
set -e
clang++ -c main.cpp -o main.o
clang++ -c runtime.cpp -o runtime.o
clang++ main.o runtime.o -o a.out
./a.out
