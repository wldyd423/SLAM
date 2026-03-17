# Step 4 — Noisy Sensor Dot

## SLAM Concept
Real sensors never return the exact true position. A GPS-like sensor returns the true
position plus independent Gaussian noise on each axis:

```
measurement = (x_true + ε_x,  y_true + ε_y)
ε_x, ε_y ~ N(0, σ²)
```

`σ` (sigma) is the **standard deviation** of the noise. Small σ → readings cluster
tightly around truth. Large σ → readings scatter widely. This is the simplest possible
sensor model — no bearing, no range, just noisy coordinates.

---

## Library Reference: rand, rand_distr, and egui sidebar

### `rand` crate — generating random numbers
Add to `Cargo.toml`:
```toml
[dependencies]
rand        = "0.9"
rand_distr  = "0.5"
```

`rand::rng()` returns a thread-local RNG (random number generator). It is
cryptographically seeded automatically and cheap to call:
```rust
let mut rng = rand::rng();
```
You can call this every frame — it is essentially free.

---

### `rand_distr::Normal` — Gaussian distribution
```rust
use rand_distr::{Distribution, Normal};

// Normal::new(mean, std_dev)
// Returns Err if std_dev < 0, so unwrap() is safe when you pass a non-negative value.
let dist = Normal::new(0.0_f32, self.sigma).unwrap();

// Sample one value from the distribution:
let noise_x: f32 = dist.sample(&mut rng);
let noise_y: f32 = dist.sample(&mut rng);
```
Each call to `.sample` draws an independent random value from `N(mean, std_dev²)`.

Full sensor sample:
```rust
let dist = Normal::new(0.0_f32, self.sigma).unwrap();
let mut rng = rand::rng();
let mx = self.robot.x + dist.sample(&mut rng);
let my = self.robot.y + dist.sample(&mut rng);
self.last_sensor = Some((mx, my));
```

---

### `egui::SidePanel` — the right-hand controls sidebar
```rust
// SidePanel::right("id") — the string is an internal egui ID, must be unique.
// .show(ctx, |ui| { ... }) — builds the panel contents.
egui::SidePanel::right("controls")
    .min_width(180.0)           // optional: minimum panel width in pixels
    .show(ctx, |ui| {
        ui.heading("Controls"); // large bold label
        ui.separator();         // horizontal dividing line
        // ... widgets go here ...
    });
```
**Important:** `SidePanel` must be called *before* `CentralPanel`. egui lays out
panels from the outside in — the central panel fills whatever space is left.

---

### `ui.add` and `egui::Slider` — interactive sliders
```rust
// Slider::new(&mut value, range) — binds directly to a mutable reference.
// Changes to the slider immediately update `value`.
ui.add(egui::Slider::new(&mut self.sigma, 0.0_f32..=100.0)
    .text("Sensor σ")       // label shown next to the slider
    .suffix(" px")          // optional unit suffix displayed on the value
);
```
The slider widget mutates `self.sigma` in place. When the user drags the slider,
`self.sigma` has the new value before the canvas draws — so the effect is immediate.

Shorter form using the convenience method:
```rust
ui.add(egui::Slider::new(&mut self.sigma, 0.0..=100.0).text("Sensor σ"));
```

---

### `ui.label` — static text
```rust
ui.label("Sensor noise:");           // plain text
ui.label(egui::RichText::new("σ").monospace()); // styled text
```

---

### `ui.separator` and `ui.add_space`
```rust
ui.separator();           // horizontal rule
ui.add_space(8.0);        // vertical gap of 8 pixels
```

---

### Drawing the sensor dot
After computing `(mx, my)`, draw a small red dot on the painter:
```rust
if let Some((mx, my)) = self.last_sensor {
    painter.circle_filled(
        egui::pos2(mx, my),
        4.0,                                        // radius — smaller than robot
        egui::Color32::from_rgb(220, 60, 60),       // red
    );
}
```

---

### Full sidebar skeleton
```rust
egui::SidePanel::right("controls").min_width(180.0).show(ctx, |ui| {
    ui.heading("Controls");
    ui.separator();

    ui.label("Sensor noise");
    ui.add(egui::Slider::new(&mut self.sigma, 0.0_f32..=100.0).text("σ"));
});

// Then the canvas:
egui::CentralPanel::default().show(ctx, |ui| {
    let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
    // ... draw heatmap, sensor dot, robot ...
});
```

---

## What to Implement
1. Add `rand` and `rand_distr` to `Cargo.toml`.
2. Add `sigma: f32` (start at `15.0`) and `last_sensor: Option<(f32, f32)>` to `SlamApp`.
3. Each frame, sample a sensor reading and store it in `self.last_sensor`.
4. Draw the reading as a small red dot on the painter.
5. Add a `SidePanel::right` with a `Slider` controlling `self.sigma`.

## Visual Result
A red dot jitters around the blue robot dot. Moving the σ slider right makes the
red dot jump further from the robot. At σ ≈ 0 the two dots nearly overlap.

## Hints
- `SidePanel` before `CentralPanel` — always.
- Store `last_sensor: Option<(f32, f32)>` so later steps (trail, grid) can read the
  same value without re-sampling.
- If sigma is 0, `Normal::new(0.0, 0.0)` returns an error. Guard with:
  `let sigma = self.sigma.max(0.001);`
