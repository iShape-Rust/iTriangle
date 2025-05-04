//
// Created by Nail Sharipov on 03.05.2025.
//

#include "test_1_star_with_hole.h"
#include <cmath>
#include <chrono>
#include <iostream>

StarWithHoleTest::StarWithHoleTest(double radius,
                                   size_t angle_steps_count,
                                   size_t points_per_corner,
                                   size_t radius_steps_count,
                                   double min_radius_scale,
                                   double max_radius_scale)
        : radius(radius),
          angle_steps_count(angle_steps_count),
          points_per_corner(points_per_corner),
          radius_steps_count(radius_steps_count),
          min_radius_scale(min_radius_scale),
          max_radius_scale(max_radius_scale) {}

size_t StarWithHoleTest::run_earcut(size_t count) const {
    size_t count_per_star = points_per_corner * count;
    vector<vector<Point>> shape = {
            vector<Point>(),
            vector<Point>()
    };
    shape[0].reserve(count_per_star);
    shape[1].reserve(count_per_star);

    size_t sum = 0;

    double angle_step = 2.0 * M_PI / static_cast<double>(angle_steps_count);
    double radius_scale = min_radius_scale;
    double radius_step = (max_radius_scale - min_radius_scale) /
                         static_cast<double>(radius_steps_count);

    auto start = std::chrono::high_resolution_clock::now();

    for (size_t i = 0; i < radius_steps_count; ++i) {
        double start_angle = 0.0;
        for (size_t j = 0; j < angle_steps_count; ++j) {
            StarBuilder::fill_star_with_hole(
                    radius,
                    radius_scale,
                    start_angle,
                    points_per_corner,
                    count,
                    shape
            );
            sum += run_shape(shape);
            start_angle += angle_step;
        }
        radius_scale += radius_step;
    }

    auto end = std::chrono::high_resolution_clock::now();
    std::chrono::duration<double> duration = end - start;
    std::cout << count << " - " << duration.count() << std::endl;

    return sum;
}

size_t StarWithHoleTest::run_shape(const vector<vector<Point>>& shape) const {
    vector<uint32_t> result = mapbox::earcut<uint32_t>(shape);
    return result.size();
}

size_t StarWithHoleTest::run_triangle(size_t count) const {
    size_t count_per_star = points_per_corner * count;

    vector<double> points(count_per_star * 2 * 2);

    size_t sum = 0;

    double angle_step = 2.0 * M_PI / static_cast<double>(angle_steps_count);
    double radius_scale = min_radius_scale;
    double radius_step = (max_radius_scale - min_radius_scale) /
                         static_cast<double>(radius_steps_count);

    auto start = std::chrono::high_resolution_clock::now();

    for (size_t i = 0; i < radius_steps_count; ++i) {
        double start_angle = 0.0;
        for (size_t j = 0; j < angle_steps_count; ++j) {
            StarBuilder::fill_star_with_hole_flat(
                    radius,
                    radius_scale,
                    start_angle,
                    points_per_corner,
                    count,
                    true,
                    points
            );
            sum += run_points(points);
            start_angle += angle_step;
        }
        radius_scale += radius_step;
    }

    auto end = std::chrono::high_resolution_clock::now();
    std::chrono::duration<double> duration = end - start;
    std::cout << count << " - " << duration.count() << std::endl;

    return sum;
}

size_t StarWithHoleTest::run_points(const vector<double>& points) const {
    struct triangulateio in{};
    struct triangulateio out{};

    size_t total_point_count = points.size() / 2;
    size_t hole_start_index = total_point_count / 2;
    size_t outer_point_count = hole_start_index;
    size_t inner_point_count = total_point_count - hole_start_index;

    in.numberofpoints = static_cast<int>(total_point_count);
    in.pointlist = const_cast<double*>(points.data());

    // Segments: one per edge, outer + hole
    in.numberofsegments = static_cast<int>(total_point_count);
    in.segmentlist = (int*)malloc(sizeof(int) * total_point_count * 2);
    for (int i = 0; i < in.numberofsegments; ++i) {
        in.segmentlist[i * 2] = i;
        in.segmentlist[i * 2 + 1] = (i + 1) % total_point_count;
        if (i + 1 == hole_start_index) // close outer ring
            in.segmentlist[i * 2 + 1] = 0;
        else if (i + 1 == total_point_count) // close inner ring
            in.segmentlist[i * 2 + 1] = hole_start_index;
    }

    // One hole
    in.numberofholes = 1;
    in.holelist = (double*)malloc(sizeof(double) * 2);
    {
        // Compute hole interior point (centroid of inner ring)
        double cx = 0.0, cy = 0.0;
        for (size_t i = hole_start_index; i < total_point_count; ++i) {
            cx += points[i * 2];
            cy += points[i * 2 + 1];
        }
        size_t inner_count = total_point_count - hole_start_index;
        in.holelist[0] = cx / inner_count;
        in.holelist[1] = cy / inner_count;
    }

    triangulate((char*)"pzQ", &in, &out, nullptr);
    size_t triangle_count = out.numberoftriangles;

    // Clean up
    free(out.pointlist);
    free(out.trianglelist);
    free(in.segmentlist);
    free(in.holelist);

    return triangle_count * 3;
}