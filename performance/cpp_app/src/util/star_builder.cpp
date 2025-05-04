#include "star_builder.h"
#include <cmath>

void StarBuilder::fill_star(
        double radius,
        double radius_scale,
        double start_angle,
        size_t points_per_corner,
        size_t corners_count,
        bool direction,
        vector<Point>& points
) {
    points.clear();
    fill_star_contour(
            {0.0, 0.0},
            radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            direction,
            points
    );
}

void StarBuilder::fill_star_with_hole(
        double radius,
        double radius_scale,
        double start_angle,
        size_t points_per_corner,
        size_t corners_count,
        vector<vector<Point>>& contours
) {
    contours[0].clear();
    contours[1].clear();

    fill_star_contour(
            {0.0, 0.0},
            radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            true,
            contours[0]
    );

    fill_star_contour(
            {0.0, 0.0},
            0.5 * radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            true,
            contours[1]
    );
}

void StarBuilder::fill_star_contour(
        const std::array<double, 2>& center,
        double radius,
        double radius_scale,
        double start_angle,
        size_t points_per_corner,
        size_t corners_count,
        bool direction,
        vector<Point>& points
) {
    size_t points_count = points_per_corner * corners_count;
    double sign = direction ? 1.0 : -1.0;
    double da = sign * 2.0 * M_PI / static_cast<double>(points_count);
    double w = static_cast<double>(corners_count);
    double a = 0.0;

    for (size_t i = 0; i < points_count; ++i) {
        double r = radius * (1.0 + radius_scale * std::cos(w * a));
        double angle = a + start_angle;
        double x = r * std::cos(angle) + center[0];
        double y = r * std::sin(angle) + center[1];
        a += da;
        points.push_back({x, y});
    }
}

void StarBuilder::fill_star_contour_flat(
        const std::array<double, 2>& center,
        double radius,
        double radius_scale,
        double start_angle,
        size_t points_per_corner,
        size_t corners_count,
        bool direction,
        vector<double>& points
) {
    size_t points_count = points_per_corner * corners_count;
    double sign = direction ? 1.0 : -1.0;
    double da = sign * 2.0 * M_PI / static_cast<double>(points_count);
    double w = static_cast<double>(corners_count);
    double a = 0.0;

    for (size_t i = 0; i < points_count; ++i) {
        double r = radius * (1.0 + radius_scale * std::cos(w * a));
        double angle = a + start_angle;
        double x = r * std::cos(angle) + center[0];
        double y = r * std::sin(angle) + center[1];
        a += da;

        points.push_back(x);
        points.push_back(y);
    }
}

void StarBuilder::fill_star_flat(
        double radius,
        double radius_scale,
        double start_angle,
        size_t points_per_corner,
        size_t corners_count,
        bool direction,
        vector<double>& points
) {
    points.clear();
    fill_star_contour_flat(
            {0.0, 0.0},
            radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            direction,
            points
    );
}

void StarBuilder::fill_star_with_hole_flat(
        double radius,
        double radius_scale,
        double start_angle,
        size_t points_per_corner,
        size_t corners_count,
        bool direction,
        vector<double>& shape
) {
    shape.clear();

    fill_star_contour_flat(
            {0.0, 0.0},
            radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            true,
            shape
    );

    fill_star_contour_flat(
            {0.0, 0.0},
            0.5 * radius,
            radius_scale,
            start_angle,
            points_per_corner,
            corners_count,
            true,
            shape
    );
}