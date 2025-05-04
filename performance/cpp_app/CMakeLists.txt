cmake_minimum_required(VERSION 3.26)
project(cpp_app)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

file(GLOB_RECURSE SOURCES src/*.cpp)

add_library(triangle_lib OBJECT lib/triangle.c)
target_compile_definitions(triangle_lib PRIVATE TRILIBRARY ANSI_DECLARATORS)
target_include_directories(triangle_lib PRIVATE lib)
target_compile_options(triangle_lib PRIVATE -Wno-deprecated-non-prototype)

# Main executable
add_executable(cpp_app
        ${SOURCES}
        $<TARGET_OBJECTS:triangle_lib>
        src/test/test_0_star.cpp
        src/test/test_0_star.h
        src/util/args.cpp
        src/util/args.h
        src/util/star_builder.cpp
        src/util/star_builder.h
        src/test/test_1_star_with_hole.cpp
        src/test/test_1_star_with_hole.h
        src/test/test_2_rect_star_holes.cpp
        src/test/test_2_rect_star_holes.h
        src/util/triangle_wrapper.h
)

target_include_directories(cpp_app PRIVATE lib src)
