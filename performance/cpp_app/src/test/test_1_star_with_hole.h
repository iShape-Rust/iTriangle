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
#include "util/triangle_wrapper.h"

/*
earcut:

4 - 0.136161
8 - 0.454629
16 - 1.52663
32 - 5.16188
64 - 25.1207
128 - 161.923

triangle:

4 - 0.801385
8 - 1.59074
16 - 3.12989
32 - 6.21763
64 - 12.29
128 - 24.6715
256 - 50.5745

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
    size_t run_triangle(size_t count) const;
private:
    size_t run_shape(const vector<vector<Point>>& shape) const;
    size_t run_points(const vector<double>& points) const;
};


#endif //CPP_APP_TEST_1_STAR_WITH_HOLE_H
