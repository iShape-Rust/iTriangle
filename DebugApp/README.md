# iTriangle debug applications

`uniform_grid` is an interactive visual check for `UniformTriangulatable` and
`IntUniformGrid`. It shows the editable input boundary, the separately colored
boundary resampled by `SliceContour`, the protective inner offset, uniform
Steiner points, and the resulting Delaunay mesh. Every resampled boundary edge
is at most `edge_length` long.

Run it from the iTriangle repository root:

```sh
cargo run --manifest-path DebugApp/uniform_grid/Cargo.toml
```

Drag a yellow vertex to edit the active contour. Use the mouse wheel to zoom,
and the middle or right mouse button to pan.
