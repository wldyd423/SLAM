# Step 6 — 2D Gaussian Overlay (Analytical)

## SLAM Concept
Given a sensor measurement `(mx, my)` with known noise `σ`, the **likelihood** that
the robot is at grid cell `(gx, gy)` is:

```
L(gx, gy | mx, my) = exp( -[(gx-mx)² + (gy-my)²] / (2σ²) )
```

This is the 2D Gaussian (unnormalized). It is highest at the measurement center and
falls off with distance. Visualizing it as a heatmap gives one snapshot of "where
could the robot be given this one reading?"

This step introduces the **likelihood function** — one of the two key ingredients
of Bayes' rule (`posterior ∝ likelihood × prior`).

---

## Library Reference: flat 2-D grids and painting rectangles

### Storing a 2-D grid as a flat `Vec<f32>`
A `Vec<f32>` of length `W * H` stores a 2-D grid in row-major order. To access
cell at `(row, col)`:
```rust
const GRID_W: usize = 100;
const GRID_H: usize = 100;

let mut grid = vec![0.0_f32; GRID_W * GRID_H];

// Write cell (row=3, col=7):
grid[3 * GRID_W + 7] = 1.0;

// Read cell (row, col):
let v = grid[row * GRID_W + col];
```
This is faster than a `Vec<Vec<f32>>` (no pointer indirection) and maps naturally
to image buffers.

---

### Computing the Gaussian likelihood over the grid
```rust
// Called once per sensor reading (mx, my):
fn compute_likelihood(
    likelihood: &mut Vec<f32>,
    mx: f32, my: f32,
    sigma: f32,
    canvas_w: f32, canvas_h: f32,
) {
    let two_sigma_sq = 2.0 * sigma * sigma;
    for row in 0..GRID_H {
        for col in 0..GRID_W {
            // Map grid indices → world (canvas pixel) coordinates
            let wx = col as f32 / GRID_W as f32 * canvas_w;
            let wy = row as f32 / GRID_H as f32 * canvas_h;
            let dx = wx - mx;
            let dy = wy - my;
            likelihood[row * GRID_W + col] = (-(dx*dx + dy*dy) / two_sigma_sq).exp();
            // f32::exp(x) = eˣ  — values in (0, 1] for x ≤ 0
        }
    }
}
```
`f32::exp` is the natural exponential. For `x = 0` (at the measurement center) it
returns `1.0`. For large negative `x` (far from center) it approaches `0.0`.

---

### Mapping a value to a color (blue → red gradient)
```rust
fn value_to_color(v: f32) -> egui::Color32 {
    // v is in [0.0, 1.0]
    // 0.0 → pure blue,  1.0 → pure red,  0.5 → equal mix
    let r = (v * 255.0) as u8;
    let b = ((1.0 - v) * 255.0) as u8;
    egui::Color32::from_rgba_unmultiplied(r, 0, b, 120)
    //                                              ^^^ semi-transparent so robot stays visible
}
```
You can also interpolate through more colors (e.g. blue → cyan → green → yellow → red)
for a more vivid heatmap, but the two-color version is enough to start.

---

### `painter.rect_filled` — draw a solid rectangle
```rust
painter.rect_filled(
    rect,   // egui::Rect — the area to fill
    0.0,    // corner rounding radius (0 = sharp corners)
    color,  // egui::Color32
);
```

### `egui::Rect` — constructing rectangles
```rust
// From top-left corner + size:
let rect = egui::Rect::from_min_size(
    egui::pos2(x, y),              // top-left corner
    egui::vec2(cell_w, cell_h),    // width, height
);

// From two corners:
let rect = egui::Rect::from_min_max(
    egui::pos2(x0, y0),   // top-left
    egui::pos2(x1, y1),   // bottom-right
);
```

---

### Drawing the heatmap grid
```rust
let cell_w = canvas_rect.width()  / GRID_W as f32;
let cell_h = canvas_rect.height() / GRID_H as f32;

// Find max value for relative scaling (makes colors vivid even when values are tiny):
let max_val = likelihood.iter().cloned().fold(0.0_f32, f32::max);
if max_val > 0.0 {
    for row in 0..GRID_H {
        for col in 0..GRID_W {
            let v = likelihood[row * GRID_W + col] / max_val;  // normalize to [0,1]
            let color = value_to_color(v);

            let x = canvas_rect.min.x + col as f32 * cell_w;
            let y = canvas_rect.min.y + row as f32 * cell_h;
            let rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_w, cell_h));

            painter.rect_filled(rect, 0.0, color);
        }
    }
}
```
**Draw the grid first** (before sensor dots and robot) so everything else appears on top.

---

### Getting the canvas rect from `allocate_painter`
```rust
egui::CentralPanel::default().show(ctx, |ui| {
    let (response, painter) = ui.allocate_painter(
        ui.available_size(),
        egui::Sense::hover(),
    );
    let canvas_rect = response.rect;
    // canvas_rect.min  → top-left Pos2
    // canvas_rect.max  → bottom-right Pos2
    // canvas_rect.width()  / .height()  → dimensions as f32
});
```

---

## What to Implement
1. Add `likelihood: Vec<f32>` (length `GRID_W * GRID_H`) to `SlamApp`.
2. Each frame, recompute the likelihood from the latest sensor reading.
3. Draw the grid as colored rectangles (heatmap) before drawing dots and robot.
4. Create `src/filter/mod.rs` and `src/filter/grid.rs` and move grid logic there.

## Visual Result
Each sensor reading produces a Gaussian blob on the canvas — red hot at the
measurement center, fading to blue at the edges. High σ → wide dim blob.
Low σ → sharp bright spot.

## Hints
- `O(100×100) = 10 000` iterations per frame is fast. Only optimize if you see lag.
- `canvas_rect.min.x / .y` offset the grid correctly even when a sidebar is present —
  the canvas does not start at `(0, 0)` once a `SidePanel` exists.
- `egui::Rect::from_min_size` is the easiest constructor for grid cells.
