#include "test_0_star.h"


SimpleStarTest::SimpleStarTest(double radius,
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

size_t SimpleStarTest::run_earcut(size_t count) const {
    size_t count_per_star = points_per_corner * count;
    vector<vector<Point>> shape = {
            vector<Point>()
    };
    shape[0].reserve(count_per_star);

    size_t sum = 0;

    double angle_step = 2.0 * M_PI / static_cast<double>(angle_steps_count);
    double radius_scale = min_radius_scale;
    double radius_step = (max_radius_scale - min_radius_scale) /
                         static_cast<double>(radius_steps_count);

    auto start = std::chrono::high_resolution_clock::now();

    for (size_t i = 0; i < radius_steps_count; ++i) {
        double start_angle = 0.0;
        for (size_t j = 0; j < angle_steps_count; ++j) {
            StarBuilder::fill_star(
                    radius,
                    radius_scale,
                    start_angle,
                    points_per_corner,
                    count,
                    true,
                    shape[0]
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

size_t SimpleStarTest::run_shape(const vector<vector<Point>>& shape) const {
    vector<uint32_t> result = mapbox::earcut<uint32_t>(shape);
    return result.size();
}