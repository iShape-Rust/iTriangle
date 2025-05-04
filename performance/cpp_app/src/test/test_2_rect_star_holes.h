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
#include "util/triangle_wrapper.h"

/*
earcut:

4 - 0.00531192
8 - 0.0431032
16 - 0.571262
32 - 8.54836
64 - 201.28

triangle:
4 - 0.0281862
8 - 0.094576
16 - 0.329305
32 - 1.34492
64 - 5.86257
128 - 28.4693
256 - 175.662

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

    size_t run_triangle(size_t count) const;

private:
    void fill_rect_shape(double radius_scale,
                         double start_angle,
                         size_t count,
                         vector<vector<Point>> &shape) const;

    size_t run_shape(const vector<vector<Point>> &shape) const;

    void fill_rect_shape_flat(double radius_scale,
                              double start_angle,
                              size_t count,
                              vector<double> &shape) const;

    size_t run_points(const vector<double>& points, size_t count) const;
};

#endif //CPP_APP_TEST_2_RECT_STAR_HOLES_H
