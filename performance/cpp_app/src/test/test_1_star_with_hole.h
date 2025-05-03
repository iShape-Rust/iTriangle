//
// Created by Nail Sharipov on 03.05.2025.
//

#ifndef CPP_APP_TEST_1_STAR_WITH_HOLE_H
#define CPP_APP_TEST_1_STAR_WITH_HOLE_H

#include <cstddef>
#include <vector>
#include <array>
#include <iostream>
#include "util/star_builder.h"
#include "../lib/earcut.hpp"

/*
4 - 0.136161
8 - 0.454629
16 - 1.52663
32 - 5.16188
64 - 25.1207
128 - 161.923

 */

using namespace std;
using Point = array<double, 2>;

class StarWithHoleTest {
public:
    double radius;
    size_t angle_steps_count;
    size_t points_per_corner;
    size_t radius_steps_count;
    double min_radius_scale;
    double max_radius_scale;

    StarWithHoleTest(double radius,
                     size_t angle_steps_count,
                     size_t points_per_corner,
                     size_t radius_steps_count,
                     double min_radius_scale,
                     double max_radius_scale);

    size_t run_earcut(size_t count) const;

private:
    size_t run_shape(const vector<vector<Point>>& shape) const;
};


#endif //CPP_APP_TEST_1_STAR_WITH_HOLE_H
