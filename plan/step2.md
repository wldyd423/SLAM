# Step 2 — Robot on Canvas

## SLAM Concept
The robot has a **true pose** `(x, y, θ)` — the ground-truth state we will eventually
try to estimate. In real SLAM this pose is hidden; here we display it directly so we
can compare truth vs. belief throughout later steps.

`θ` (theta) is the heading angle in radians, measured counter-clockwise from the
positive-x axis. It drives the direction the robot moves and the direction the heading
arrow points.

---

## Library Reference: egui drawing

### Coordinate system
egui uses **screen coordinates**: `(0, 0)` is the top-left corner of the window.
- `x` increases to the right
- `y` increases **downward** (opposite of standard math convention)

This means `sin(θ)` points *down* on screen when `θ` is positive. It looks fine
visually — just keep it in mind if angles seem mirrored.

---

### `egui::Pos2` and `egui::Vec2`
```rust
// Pos2 = a 2-D point (absolute position on screen)
let center = egui::Pos2::new(400.0, 300.0);

// Vec2 = a 2-D offset / direction (relative)
let offset = egui::Vec2::new(20.0, 0.0);

// Adding them gives a new Pos2:
let tip = center + offset;   // Pos2 { x: 420.0, y: 300.0 }

// Shorthand constructor (returns Vec2):
let v = egui::vec2(1.0, 0.0);
// Shorthand constructor (returns Pos2):
let p = egui::pos2(400.0, 300.0);
```

---

### `egui::Color32` — colors
```rust
// Opaque color from RGB bytes (0-255 each):
let blue  = egui::Color32::from_rgb(50, 120, 220);
let red   = egui::Color32::from_rgb(220, 60, 60);

// Semi-transparent (last arg = alpha, 0 = invisible, 255 = opaque):
let ghost = egui::Color32::from_rgba_unmultiplied(255, 0, 0, 80);

// Built-in named colors:
let white = egui::Color32::WHITE;
let black = egui::Color32::BLACK;
```

---

### Getting a `Painter` from `CentralPanel`
The `Painter` is your low-level drawing handle. You get it from the UI region:
```rust
egui::CentralPanel::default().show(ctx, |ui| {
    // Allocate the full panel area as an interactive region.
    // `Sense::hover()` means: track mouse hover but don't consume clicks.
    let (response, painter) = ui.allocate_painter(
        ui.available_size(),   // use all remaining space in this panel
        egui::Sense::hover(),
    );
    // `response.rect` is the screen Rect this region occupies.
    // `painter` is valid for the lifetime of this closure.
    let canvas_rect = response.rect;
});
```
Alternatively, `ui.painter()` gives a painter for the whole UI area without
allocating a specific rect — fine for simple cases.

---

### `painter.circle_filled` — draw a solid circle
```rust
painter.circle_filled(
    center,  // egui::Pos2 — center of the circle
    radius,  // f32 — radius in pixels
    color,   // egui::Color32
);
// Example:
painter.circle_filled(egui::pos2(400.0, 300.0), 8.0, egui::Color32::from_rgb(50, 150, 255));
```

There is also `painter.circle_stroke` for an outline-only circle:
```rust
painter.circle_stroke(center, radius, egui::Stroke::new(2.0, egui::Color32::WHITE));
//                                                        ^^^ line width in px
```

---

### `painter.line_segment` — draw a line between two points
```rust
painter.line_segment(
    [point_a, point_b],               // [Pos2; 2] — start and end
    egui::Stroke::new(2.0, color),    // line width + color
);
// Example — heading arrow:
let tip = center + egui::vec2(20.0 * theta.cos(), 20.0 * theta.sin());
painter.line_segment([center, tip], egui::Stroke::new(2.0, egui::Color32::WHITE));
```

---

### Putting it all together — drawing the robot
```rust
// Inside the CentralPanel closure, after getting `painter` and `canvas_rect`:

let center = egui::pos2(self.robot.x, self.robot.y);
let theta  = self.robot.theta;

// Body
painter.circle_filled(center, 8.0, egui::Color32::from_rgb(50, 150, 255));

// Heading arrow (length 20 px in direction theta)
let arrow_len = 20.0;
let tip = center + egui::vec2(arrow_len * theta.cos(), arrow_len * theta.sin());
painter.line_segment([center, tip], egui::Stroke::new(2.0, egui::Color32::WHITE));
```

---

## What to Implement
1. Add `struct Robot { x: f32, y: f32, theta: f32 }` to `main.rs` (or `robot.rs`).
2. Add `robot: Robot` to `SlamApp`, initialized near the canvas center.
3. In `update`, use `ui.allocate_painter` inside `CentralPanel` to get a painter.
4. Draw the robot body as a filled circle.
5. Draw the heading arrow as a line segment.

## Visual Result
The window shows a blue dot with a short white line pointing in the initial heading
direction (rightward at `θ = 0`).

## Hints
- World units = screen pixels for now. Place the robot at roughly `(500, 350)` or
  use a hardcoded center until you have a real canvas rect.
- The heading endpoint formula: `center + vec2(L * theta.cos(), L * theta.sin())`
- `egui::Pos2` does not implement `*` with scalars — use `Vec2` for offsets and
  add them to a `Pos2`.
