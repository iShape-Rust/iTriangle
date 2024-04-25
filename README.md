# iTriangle
A fast and efficient library for Delaunay triangulation and converting complex polygons into convex shapes, including advanced self-intersection resolution.
 
## Delaunay triangulation
<p align="center">
 <img src="readme/star_triangle.svg" width="500"/>
</p>

## Breaking into convex polygons
<p align="center">
 <img src="readme/star_polygon.svg" width="500"/>
</p>

## Features

- **Delaunay Triangulation**: *Efficient and robust implementation for generating Delaunay triangulations.*
- **Convex Polygons**: *Break complex polygons into simpler convex polygons.*
- **Self-Intersection**: *Smart intersection resolution with Even-Odd or Non-Zero rules.*

## Documentation
- [Delaunay](https://ishape-rust.github.io/iShape-js/triangle/delaunay.html)

## Getting Started

Add the following to your Cargo.toml:
```
[dependencies]
i_float = "^1.0.0"
i_overlay = "^1.0.0"
```

After that, represent your polygon as an array of vertices. Here's an example of a cheese polygon:

<p align="center">
 <img src="readme/cheese_example.svg" width="600"/>
</p>

```rust
let shape = [
    [ // body
        F64Point::new(0.0, 20.0),    // 0
        F64Point::new(8.0, 10.0),    // 1
        F64Point::new(7.0, 6.0),     // 2
        F64Point::new(9.0, 1.0),     // 3
        F64Point::new(13.0, -1.0),   // 4
        F64Point::new(17.0, 1.0),    // 5
        F64Point::new(26.0, -7.0),   // 6
        F64Point::new(14.0, -15.0),  // 7
        F64Point::new(0.0, -18.0),   // 8
        F64Point::new(-14.0, -15.0), // 9
        F64Point::new(-25.0, -7.0),  // 10
        F64Point::new(-18.0, 0.0),   // 11
        F64Point::new(-16.0, -3.0),  // 12
        F64Point::new(-13.0, -4.0),  // 13
        F64Point::new(-8.0, -2.0),   // 14
        F64Point::new(-6.0, 2.0),    // 15
        F64Point::new(-7.0, 6.0),    // 16
        F64Point::new(-10.0, 8.0)    // 17
    ].to_vec(),
    [ // hole
        F64Point::new(2.0, 0.0),     // 18
        F64Point::new(-2.0, -2.0),   // 19
        F64Point::new(-4.0, -5.0),   // 20
        F64Point::new(-2.0, -9.0),   // 21
        F64Point::new(2.0, -11.0),   // 22
        F64Point::new(5.0, -9.0),    // 23
        F64Point::new(7.0, -5.0),    // 24
        F64Point::new(5.0, -2.0)     // 25
    ].to_vec()
].to_vec();

let triangulation = shape.to_triangulation(Some(FillRule::NonZero), 0.0);

println!("points: {:?}", triangulation.points);
println!("indices: {:?}", triangulation.indices);
```

**Output Triangulation**: *triangles indices and vertices, where all triangles oriented in a clockwise direction.*
