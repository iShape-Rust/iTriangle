#include <iostream>
#include "util/args.h"
#include "test/test_0_star.h"
#include "test/test_1_star_with_hole.h"
#include "test/test_2_rect_star_holes.h"

void star(const EnvArgs& args);
void star_with_hole(const EnvArgs& args);
void rect_with_star_holes(const EnvArgs& args);

int main(int argc, char** argv) {
    EnvArgs args(argc, argv);
    size_t test = args.get_usize("test", 0);

    switch (test) {
        case 0: star(args); break;
        case 1: star_with_hole(args); break;
        case 2: rect_with_star_holes(args); break;
        default: std::cerr << "Unknown test\n"; return 1;
    }
}

void star(const EnvArgs& args) {
    SimpleStarTest test(
            100.0,
            100,
            10,
            100,
            0.0,
            1.0
    );

    bool complex = args.get_bool("complex");

    if (complex) {
        std::cout << "earcut: " << std::endl;
        size_t s0 = 0;
        for (int i = 0; i < 5; ++i) {
            size_t count = 4 << i;
            s0 += test.run_earcut(count);
        }

        std::cout << "s0: " << s0 << std::endl;
    } else {
        size_t count = args.get_usize("count");

        std::cout << "earcut: " << std::endl;
        size_t s0 = test.run_earcut(count);
        std::cout << std::endl;

        std::cout << "s0: " << s0 << std::endl;
    }
}

void star_with_hole(const EnvArgs& args) {
    StarWithHoleTest test(
            100.0,
            100,
            10,
            100,
            0.1, // must be > 0 to prevent intersection!
            1.0
    );

    bool complex = args.get_bool("complex");

    if (complex) {
        std::cout << "earcut: " << std::endl;
        size_t s0 = 0;
        for (int i = 0; i < 5; ++i) {
            size_t count = 4 << i;
            s0 += test.run_earcut(count);
        }

        std::cout << "s0: " << s0 << std::endl;
    } else {
        size_t count = args.get_usize("count");

        std::cout << "earcut: " << std::endl;
        size_t s0 = test.run_earcut(count);
        std::cout << std::endl;

        std::cout << "s0: " << s0 << std::endl;
    }
}

void rect_with_star_holes(const EnvArgs& args) {
    RectStarHolesTest test(
            100.0,
            5,
            10,
            5,
            0.0,
            1.0,
            5
    );

    bool complex = args.get_bool("complex");

    if (complex) {
        std::cout << "earcut: " << std::endl;
        size_t s0 = 0;
        for (int i = 0; i < 8; ++i) {
            size_t count = 4 << i;
            s0 += test.run_earcut(count);
        }

        std::cout << "s0: " << s0 << std::endl;
    } else {
        size_t count = args.get_usize("count");

        std::cout << "earcut: " << std::endl;
        size_t s0 = test.run_earcut(count);
        std::cout << std::endl;

        std::cout << "s0: " << s0 << std::endl;
    }
}