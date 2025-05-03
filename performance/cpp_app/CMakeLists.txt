cmake_minimum_required(VERSION 3.26)
project(cpp_app)

set(CMAKE_CXX_STANDARD 17)

file(GLOB_RECURSE SOURCES
        src/*.cpp
)

add_executable(cpp_app ${SOURCES}
        src/test/test_0_star.cpp
        src/test/test_0_star.h
        src/util/args.cpp
        src/util/args.h
        src/util/star_builder.cpp
        src/util/star_builder.h
        src/test/test_1_star_with_hole.cpp
        src/test/test_1_star_with_hole.h
        src/test/test_2_rect_star_holes.cpp
        src/test/test_2_rect_star_holes.h)
target_include_directories(cpp_app PRIVATE lib src)