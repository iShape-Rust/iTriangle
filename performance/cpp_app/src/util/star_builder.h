#ifndef STAR_BUILDER_H
#define STAR_BUILDER_H

#include <vector>
#include <array>
#include <cstddef>

using namespace std;
using Point = array<double, 2>;

class StarBuilder {
public:
    static void fill_star(
            double radius,
            double radius_scale,
            double start_angle,
            size_t points_per_corner,
            size_t corners_count,
            bool direction,
            vector<Point>& contour
    );

    static void fill_star_with_hole(
            double radius,
            double radius_scale,
            double start_angle,
            size_t points_per_corner,
            size_t corners_count,
            vector<vector<Point>>& contours
    );

    static void fill_star_contour(
            const std::array<double, 2>& center,
            double radius,
            double radius_scale,
            double start_angle,
            size_t points_per_corner,
            size_t corners_count,
            bool direction,
            vector<Point>& contour
    );

    static void fill_star_flat(
            double radius,
            double radius_scale,
            double start_angle,
            size_t points_per_corner,
            size_t corners_count,
            bool direction,
            vector<double>& points
    );

    static void fill_star_with_hole_flat(
            double radius,
            double radius_scale,
            double start_angle,
            size_t points_per_corner,
            size_t corners_count,
            bool direction,
            vector<double>& shape
    );

    static void fill_star_contour_flat(
            const std::array<double, 2>& center,
            double radius,
            double radius_scale,
            double start_angle,
            size_t points_per_corner,
            size_t corners_count,
            bool direction,
            vector<double>& points
    );
};

#endif // STAR_BUILDER_H
