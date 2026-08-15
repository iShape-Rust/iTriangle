# iTriangle debug applications

`uniform_grid` is an interactive visual check for `UniformTriangulatable` and
`IntUniformGrid`. It shows the editable input boundary, the separately colored
boundary resampled by `SliceContour`, lattice candidates after point
containment, points remaining after the `edge_length / 3` edge-clearance
filter, and the resulting Delaunay mesh. Every resampled boundary edge is at
most `edge_length` long. Optional centroid-net relaxation can be enabled in the
sidebar; its iteration limit defaults to 40.

Run it from the iTriangle repository root:

```sh
cargo run --manifest-path DebugApp/uniform_grid/Cargo.toml
```

Drag a yellow vertex to edit the active contour. Use the mouse wheel to zoom,
and the middle or right mouse button to pan.
