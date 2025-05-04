//
// Created by Nail Sharipov on 03.05.2025.
//

#include "test_2_rect_star_holes.h"
#include <cmath>
#include <chrono>
#include <iostream>

RectStarHolesTest::RectStarHolesTest(double radius,
                                     size_t angle_steps_count,
                                     size_t points_per_corner,
                                     size_t radius_steps_count,
                                     double min_radius_scale,
                                     double max_radius_scale,
                                     size_t corners_count)
        : radius(radius),
          angle_steps_count(angle_steps_count),
          points_per_corner(points_per_corner),
          radius_steps_count(radius_steps_count),
          min_radius_scale(min_radius_scale),
          max_radius_scale(max_radius_scale),
          corners_count(corners_count) {}

size_t RectStarHolesTest::run_earcut(size_t count) const {
    size_t count_per_star = points_per_corner * corners_count;
    vector<vector<Point>> shape(count * count + 1, vector<Point>());
    for (auto& s : shape) {
        s.reserve(count_per_star);
    }

    size_t sum = 0;

    double angle_step = 2.0 * M_PI / static_cast<double>(angle_steps_count);
    double radius_scale = min_radius_scale;
    double radius_step = (max_radius_scale - min_radius_scale) /
                         static_cast<double>(radius_steps_count);

    auto start = std::chrono::high_resolution_clock::now();

    while (radius_scale < max_radius_scale) {
        double start_angle = 0.0;
        for (size_t j = 0; j < angle_steps_count; ++j) {
            fill_rect_shape(radius_scale, start_angle, count, shape);
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

void RectStarHolesTest::fill_rect_shape(double radius_scale,
                                        double start_angle,
                                        size_t count,
                                        vector<vector<Point>>& shape) const {
    double dx = 4.0 * radius;
    double dy = dx;

    double w = dx * static_cast<double>(count);
    double h = w;

    auto& rect = shape[0];
    rect.clear();
    rect.push_back({0.0, 0.0});
    rect.push_back({w, 0.0});
    rect.push_back({w, h});
    rect.push_back({0.0, h});

    double x = 0.5 * dx;
    size_t i = 1;

    for (size_t xi = 0; xi < count; ++xi) {
        double y = 0.5 * dy;
        for (size_t yi = 0; yi < count; ++yi) {
            auto& contour = shape[i++];
            contour.clear();
            StarBuilder::fill_star_contour(
                    {x, y},
                    radius,
                    radius_scale,
                    start_angle,
                    points_per_corner,
                    corners_count,
                    true,
                    contour
            );
            y += dy;
        }
        x += dx;
    }
}

size_t RectStarHolesTest::run_shape(const vector<vector<Point>>& shape) const {
    vector<uint32_t> result = mapbox::earcut<uint32_t>(shape);
    return result.size();
}

size_t RectStarHolesTest::run_triangle(size_t count) const {
    size_t count_per_star = points_per_corner * corners_count;
    vector<double> shape;
    shape.reserve((count * count + 1) * count_per_star * 2); // outer + holes

    size_t sum = 0;

    double angle_step = 2.0 * M_PI / static_cast<double>(angle_steps_count);
    double radius_scale = min_radius_scale;
    double radius_step = (max_radius_scale - min_radius_scale) /
                         static_cast<double>(radius_steps_count);

    auto start = std::chrono::high_resolution_clock::now();

    while (radius_scale < max_radius_scale) {
        double start_angle = 0.0;
        for (size_t j = 0; j < angle_steps_count; ++j) {
            fill_rect_shape_flat(radius_scale, start_angle, count, shape);
            sum += run_points(shape, count);
            start_angle += angle_step;
        }
        radius_scale += radius_step;
    }

    auto end = std::chrono::high_resolution_clock::now();
    std::chrono::duration<double> duration = end - start;
    std::cout << count << " - " << duration.count() << std::endl;

    return sum;
}

void RectStarHolesTest::fill_rect_shape_flat(double radius_scale,
                                             double start_angle,
                                             size_t count,
                                             vector<double>& shape) const {
    shape.clear();

    double dx = 4.0 * radius;
    double dy = dx;
    double w = dx * static_cast<double>(count);
    double h = w;

    // Outer rectangle
    shape.push_back(0.0); shape.push_back(0.0);
    shape.push_back(w);   shape.push_back(0.0);
    shape.push_back(w);   shape.push_back(h);
    shape.push_back(0.0); shape.push_back(h);

    double x = 0.5 * dx;
    for (size_t xi = 0; xi < count; ++xi) {
        double y = 0.5 * dy;
        for (size_t yi = 0; yi < count; ++yi) {
            StarBuilder::fill_star_contour_flat(
                    {x, y},
                    radius,
                    radius_scale,
                    start_angle,
                    points_per_corner,
                    corners_count,
                    true,
                    shape
            );
            y += dy;
        }
        x += dx;
    }
}

size_t RectStarHolesTest::run_points(const vector<double>& points, size_t count) const {
    struct triangulateio in{};
    struct triangulateio out{};

    size_t outer_point_count = 4;
    size_t inner_count = count * count;
    size_t inner_point_count = points_per_corner * corners_count;
    size_t total_point_count = outer_point_count + inner_count * inner_point_count;

    in.numberofpoints = static_cast<int>(total_point_count);
    in.pointlist = const_cast<double*>(points.data());

    // Segments
    in.numberofsegments = static_cast<int>(total_point_count);
    in.segmentlist = (int*)malloc(sizeof(int) * total_point_count * 2);
    for (int i = 0; i < in.numberofsegments; ++i) {
        in.segmentlist[i * 2] = i;
        if (i + 1 == outer_point_count) {
            in.segmentlist[i * 2 + 1] = 0; // close outer
        } else if ((i - outer_point_count + 1) % inner_point_count == 0) {
            in.segmentlist[i * 2 + 1] = outer_point_count + (i - outer_point_count) / inner_point_count * inner_point_count;
        } else {
            in.segmentlist[i * 2 + 1] = i + 1;
        }
    }

    // Holes
    in.numberofholes = static_cast<int>(inner_count);
    in.holelist = (double*)malloc(sizeof(double) * inner_count * 2);
    for (size_t i = 0; i < inner_count; ++i) {
        size_t start = outer_point_count + i * inner_point_count;
        double cx = 0.0, cy = 0.0;
        for (size_t j = 0; j < inner_point_count; ++j) {
            cx += points[(start + j) * 2];
            cy += points[(start + j) * 2 + 1];
        }
        cx /= static_cast<double>(inner_point_count);
        cy /= static_cast<double>(inner_point_count);
        in.holelist[i * 2] = cx;
        in.holelist[i * 2 + 1] = cy;
    }

    triangulate((char*)"pzQ", &in, &out, nullptr);
    size_t triangle_count = out.numberoftriangles;

    free(out.pointlist);
    free(out.trianglelist);
    free(in.segmentlist);
    free(in.holelist);

    return triangle_count * 3;
}
