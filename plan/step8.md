# Step 8 — Predict Step (Motion Smearing)

## SLAM Concept
When the robot moves, our **uncertainty about its position grows**. The belief should
spread out to reflect that we no longer know exactly where the robot went. This is
the **predict step** of the Bayes filter:

```
belief_predicted(x') = ∫ p(x' | x, u) · belief(x) dx
```

For a Gaussian motion model, this integral is a **convolution** of the belief grid
with a Gaussian kernel whose width matches the motion uncertainty. In practice, a
simple blur approximation works well visually.

Without this step the belief stays sharp at the old position even as the robot
moves away — the heatmap "lags" behind truth. With it, the heatmap expands on each
move and then contracts again when sensor readings correct it. This expansion →
contraction cycle is the heartbeat of every Bayes filter.

---

## Library Reference: `key_pressed`, 2D array access, and the blur algorithm

### `key_pressed` vs `key_down` — fire once per press
```rust
// key_down  → true every frame while held (used in Step 3 for smooth motion)
// key_pressed → true only on the frame the key transitions from up to down

let moved = ctx.input(|i| {
    i.key_pressed(egui::Key::ArrowUp)   ||
    i.key_pressed(egui::Key::ArrowDown) ||
    i.key_pressed(egui::Key::ArrowLeft) ||
    i.key_pressed(egui::Key::ArrowRight)
});

if moved {
    blur_belief(&mut self.belief, GRID_W, GRID_H, self.blur_radius);
}
```
Use `key_pressed` here so the blur fires exactly once per key press, adding one
fixed amount of uncertainty — not once per frame while the key is held.

If you prefer continuous motion (Step 3 style with `key_down`), call blur every
frame with a small radius (`1`) instead.

---

### Accessing a 2-D flat Vec with boundary clamping
When a blur samples neighbors of cell `(row, col)`, it must not go out of bounds.
Use `saturating_sub` (can't go below 0) and `.min(limit)` (can't exceed max):

```rust
let lo_col = col.saturating_sub(radius);     // max(0, col - radius)
let hi_col = (col + radius + 1).min(GRID_W); // min(W, col + radius + 1)
// Range lo_col..hi_col is always valid
```

`usize::saturating_sub(n)` returns `0` when the subtraction would underflow —
unlike `col - radius` which panics in debug mode or wraps in release mode.

---

### Separable 1-D box blur — a Gaussian approximation
A true 2-D Gaussian convolution is expensive. A box blur (average over a window)
run **once horizontally then once vertically** approximates a Gaussian well enough
for visualization. Running it three times per motion event gives an even better
approximation (central limit theorem), but once is sufficient visually.

```rust
fn blur_belief(belief: &mut Vec<f32>, w: usize, h: usize, radius: usize) {
    let mut tmp = vec![0.0_f32; w * h];

    // --- Horizontal pass: for each cell, average its row-neighbors ---
    for row in 0..h {
        for col in 0..w {
            let lo = col.saturating_sub(radius);
            let hi = (col + radius + 1).min(w);
            // Sum cells belief[row][lo..hi]
            let sum: f32 = (lo..hi).map(|c| belief[row * w + c]).sum();
            tmp[row * w + col] = sum / (hi - lo) as f32;
        }
    }

    // --- Vertical pass: for each cell, average its column-neighbors in tmp ---
    for row in 0..h {
        for col in 0..w {
            let lo = row.saturating_sub(radius);
            let hi = (row + radius + 1).min(h);
            let sum: f32 = (lo..hi).map(|r| tmp[r * w + col]).sum();
            belief[row * w + col] = sum / (hi - lo) as f32;
        }
    }
    // After this function, belief sums to ~1 (box blur is area-preserving).
    // No renormalization needed.
}
```

**Why two passes?** A single 2-D window (averaging all cells in a square) would be
O(n·r²). Two 1-D passes are O(n·r) — significantly faster for large radii.

---

### Exposing blur radius as a sidebar slider
```rust
// In the SidePanel:
ui.add(egui::Slider::new(&mut self.blur_radius, 1_usize..=8)
    .text("Blur radius"));
```

---

### Connecting motion to the predict step
```rust
// In update(), after reading input and moving the robot:
let moved = ctx.input(|i|
    i.key_pressed(egui::Key::ArrowUp)   ||
    i.key_pressed(egui::Key::ArrowDown)
    // rotations don't change position, so you can skip them here
);
if moved {
    blur_belief(&mut self.belief, GRID_W, GRID_H, self.blur_radius);
}
// Then the sensor update runs as normal — it will sharpen the belief back.
```

---

### `ui.add(egui::Slider::new(...))` with `usize` range
```rust
// Works identically to f32 sliders, just use a usize range:
ui.add(egui::Slider::new(&mut self.blur_radius, 1_usize..=8).text("Blur radius"));
```

---

## What to Implement
1. Add `blur_radius: usize` (start at `3`) to `SlamApp`.
2. Implement `blur_belief(belief, w, h, radius)` (horizontal + vertical passes).
3. Detect motion key presses (`key_pressed`) and call `blur_belief` when movement occurs.
4. Add a "Blur radius" slider in the sidebar.

## Visual Result
- Move robot forward: heatmap immediately smears/expands.
- Stay still: sensor readings accumulate and the heatmap contracts back to a sharp peak.
- The **predict (expand) → update (contract)** cycle is clearly visible.

## Hints
- Box blur is area-preserving — no need to renormalize after blurring.
- For key-held motion (Step 3's `key_down`), apply blur every frame with radius `1`
  instead of radius `3` to avoid immediate diffusion to uniform.
- If you want rotation to also add uncertainty, call blur with a smaller radius
  (e.g., `1`) on rotation key presses too.
