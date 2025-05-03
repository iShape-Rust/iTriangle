#ifndef SIMPLE_STAR_TEST_H
#define SIMPLE_STAR_TEST_H

#include <cstddef>
#include <vector>
#include <array>
#include <iostream>
#include "util/star_builder.h"
#include "../lib/earcut.hpp"

/*
 earcut:
4 - 0.025066
8 - 0.0668991
16 - 0.219485
32 - 0.494215
64 - 1.2837
128 - 3.15526
256 - 8.58501
512 - 32.9052

 */

using namespace std;
using Point = array<double, 2>;

class SimpleStarTest {
public:
    double radius;
    size_t angle_steps_count;
    size_t points_per_corner;
    size_t radius_steps_count;
    double min_radius_scale;
    double max_radius_scale;

    SimpleStarTest(double radius,
                   size_t angle_steps_count,
                   size_t points_per_corner,
                   size_t radius_steps_count,
                   double min_radius_scale,
                   double max_radius_scale);

    size_t run_earcut(size_t count) const;

private:
    size_t run_shape(const vector<vector<Point>>& shape) const;
};

#endif // SIMPLE_STAR_TEST_H