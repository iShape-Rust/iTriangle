//
// Created by Nail Sharipov on 03.05.2025.
//

#ifndef CPP_APP_TEST_2_RECT_STAR_HOLES_H
#define CPP_APP_TEST_2_RECT_STAR_HOLES_H


#include <cstddef>
#include <vector>
#include <array>
#include <iostream>
#include "util/star_builder.h"
#include "../lib/earcut.hpp"

/*
4 - 0.00531192
8 - 0.0431032
16 - 0.571262
32 - 8.54836
64 - 201.28

 */


using namespace std;
using Point = array<double, 2>;

class RectStarHolesTest {
public:
    double radius;
    size_t angle_steps_count;
    size_t points_per_corner;
    size_t radius_steps_count;
    double min_radius_scale;
    double max_radius_scale;
    size_t corners_count;

    RectStarHolesTest(double radius,
                      size_t angle_steps_count,
                      size_t points_per_corner,
                      size_t radius_steps_count,
                      double min_radius_scale,
                      double max_radius_scale,
                      size_t corners_count);

    size_t run_earcut(size_t count) const;

private:
    void fill_rect_shape(double radius_scale,
                         double start_angle,
                         size_t count,
                         vector<vector<Point>>& shape) const;

    size_t run_shape(const vector<vector<Point>>& shape) const;
};

#endif //CPP_APP_TEST_2_RECT_STAR_HOLES_H
