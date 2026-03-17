# Step 5 — Sensor Trail & Intuition

## SLAM Concept
A single noisy reading tells us little. But many readings form a **cloud** whose
center converges on the true position. This is the empirical intuition behind
Bayesian estimation: more evidence → tighter belief.

The cloud is a visual preview of what the probability distribution will look like
in Step 6. Looking at it you can already answer "where is the robot?" by finding
the dense region.

---

## Library Reference: VecDeque and drawing many points

### `std::collections::VecDeque` — a ring buffer
`VecDeque` is a double-ended queue. We use it as a **fixed-length ring buffer**:
push new readings on the back, pop old readings from the front.

```rust
use std::collections::VecDeque;

// Declare the type — stores (x, y) pairs:
let mut trail: VecDeque<(f32, f32)> = VecDeque::new();

// Add a new reading at the back:
trail.push_back((mx, my));

// Remove the oldest reading from the front if over capacity:
if trail.len() > self.trail_len {
    trail.pop_front();
}
// Result: `trail` always holds at most `trail_len` items,
// newest at the back, oldest at the front.
```

**Why VecDeque and not Vec?**
`Vec::remove(0)` is O(n) — it shifts every element down. `VecDeque::pop_front`
is O(1) — it just advances an index pointer. For a 500-item trail, the difference
is imperceptible, but `VecDeque` is the idiomatically correct tool.

---

### Iterating over `VecDeque` to draw all points
```rust
for &(x, y) in &self.trail {
    painter.circle_filled(
        egui::pos2(x, y),
        3.0,
        egui::Color32::from_rgba_unmultiplied(255, 80, 80, 60),
        // ^^^ semi-transparent red: 255 R, 80 G, 80 B, 60 alpha (0=invisible, 255=opaque)
    );
}
```
Iterating `&self.trail` yields references to each `(f32, f32)` in order from
oldest (front) to newest (back).

---

### `Color32::from_rgba_unmultiplied` — transparency
```rust
// Opaque red:
egui::Color32::from_rgb(255, 80, 80)

// Semi-transparent red (alpha = 60 out of 255 ≈ 24% opacity):
egui::Color32::from_rgba_unmultiplied(255, 80, 80, 60)
```
The "unmultiplied" means you provide straight alpha — egui handles premultiplication
internally. Alpha `0` = fully invisible, `255` = fully opaque.

For a **fade effect** (older dots dimmer), scale alpha by index:
```rust
for (i, &(x, y)) in self.trail.iter().enumerate() {
    let alpha = (i as f32 / self.trail.len() as f32 * 180.0) as u8;
    // oldest (i=0) → alpha near 0 (invisible)
    // newest (i=len-1) → alpha near 180 (fairly opaque)
    painter.circle_filled(
        egui::pos2(x, y),
        3.0,
        egui::Color32::from_rgba_unmultiplied(255, 80, 80, alpha),
    );
}
```

---

### Optional slider for trail length
```rust
// In the SidePanel:
ui.add(egui::Slider::new(&mut self.trail_len, 10_usize..=500)
    .text("Trail length"));
```
Note the range type is `usize` — match it to the field type.

---

### Draw order matters
egui's painter draws shapes in the order you call them. Later calls appear on top.
Recommended order for this step:
1. Trail dots (oldest cloud, drawn first / underneath)
2. Current sensor dot (on top of trail)
3. Robot body and heading arrow (always visible on top)

```rust
// 1. Trail
for &(x, y) in &self.trail { painter.circle_filled(...); }

// 2. Current sensor reading
if let Some((mx, my)) = self.last_sensor {
    painter.circle_filled(egui::pos2(mx, my), 5.0, egui::Color32::RED);
}

// 3. Robot
painter.circle_filled(robot_center, 8.0, egui::Color32::from_rgb(50, 150, 255));
painter.line_segment([robot_center, arrow_tip], egui::Stroke::new(2.0, egui::Color32::WHITE));
```

---

## What to Implement
1. Add `trail: VecDeque<(f32, f32)>` and `trail_len: usize` to `SlamApp`.
2. Each frame: push the sensor reading, pop if over capacity.
3. Draw all trail points before the current sensor dot and robot.
4. (Optional) Add a sidebar slider for trail length.

## Visual Result
A fading red cloud follows the robot. When the robot is still the cloud forms a dense
blob centered on the true position. High σ → large diffuse blob. Low σ → tight cluster.

## Hints
- After this step `main.rs` is getting long. Good time to split into modules:
  - `src/robot.rs` — `struct Robot`
  - `src/sensor.rs` — `struct Sensor` (sigma + last reading + trail)
  - `src/main.rs` — `SlamApp`, `update`, imports
  - Declare them in `main.rs` with `mod robot;` and `mod sensor;`.
- Uniform low alpha (e.g. 60) is simpler than per-index fading and still looks good.
