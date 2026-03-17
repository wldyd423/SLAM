# Step 7 — Bayesian Update (Running Belief)

## SLAM Concept
Step 6 showed *one* likelihood snapshot. Now we **accumulate** evidence over time
using Bayes' rule:

```
belief_new(x) ∝ likelihood(z | x) × belief_old(x)
```

In the discrete grid case this is element-wise multiplication followed by
normalization (divide each cell by the total sum so the grid sums to 1).

The belief **concentrates** around the true position as more readings arrive.
If the robot moves without a predict step (Step 8), the belief will lag behind —
you will see the heatmap stuck at the old position while the robot walks away.
That lag motivates Step 8.

---

## Library Reference: Vec arithmetic, normalization, and egui buttons

### Element-wise Vec multiplication (Bayes update)
```rust
// belief and likelihood are both Vec<f32> of the same length.
// Multiply each cell in place:
for i in 0..self.belief.len() {
    self.belief[i] *= self.likelihood[i];
}
// After this, belief[i] = old_belief[i] * likelihood[i]
// The values no longer sum to 1 — we must normalize.
```

Using iterators (same result, more idiomatic):
```rust
for (b, &l) in self.belief.iter_mut().zip(self.likelihood.iter()) {
    *b *= l;
}
```

---

### Normalization — making a probability distribution
After multiplication the cells can have any positive values. To turn them back into
a probability distribution (summing to 1), divide each cell by the total:
```rust
let sum: f32 = self.belief.iter().sum();  // sum all elements
if sum > 1e-10 {                          // guard against all-zero (degenerate case)
    for v in &mut self.belief {
        *v /= sum;
    }
} else {
    // Belief has collapsed to zero — reset to uniform prior:
    let n = self.belief.len();
    self.belief.fill(1.0 / n as f32);
}
```

**Why can it collapse to zero?** After many multiplications by small numbers,
all cells approach `f32::MIN_POSITIVE` and eventually underflow to `0.0`. The
normalization guard catches this and resets gracefully.

---

### Initializing the belief to a uniform prior
```rust
// Every cell has equal probability — "I have no idea where the robot is."
let n = GRID_W * GRID_H;
let belief = vec![1.0 / n as f32; n];
```

---

### Resetting with `Vec::fill`
```rust
// Reset belief to uniform (e.g., when user presses "Reset"):
let n = self.belief.len();
self.belief.fill(1.0 / n as f32);
// fill() sets every element to the given value. It's equivalent to:
// for v in &mut self.belief { *v = 1.0 / n as f32; }
```

---

### Relative color scaling with `fold` and `f32::max`
After many updates, the actual probability values become very small. Coloring by
absolute value makes the whole grid look dark. Scale by the maximum instead:
```rust
// f32 has no built-in max() for iterators (because f32 is not Ord).
// Use fold with f32::max as the combinator:
let max_val = self.belief.iter().cloned().fold(0.0_f32, f32::max);

// Now use v / max_val as input to your color function:
for (i, &v) in self.belief.iter().enumerate() {
    let normalized = if max_val > 0.0 { v / max_val } else { 0.0 };
    let color = value_to_color(normalized);
    // ... draw cell i ...
}
```
`fold(initial, fn)` — like `reduce` in other languages. Starts with `0.0` and
repeatedly calls `f32::max(accumulator, next_element)`.

---

### `ui.button` — a clickable button
```rust
// In the SidePanel:
if ui.button("Reset Belief").clicked() {
    // This block runs exactly once on the frame the user clicks.
    let n = self.belief.len();
    self.belief.fill(1.0 / n as f32);
}
```
`ui.button("text")` returns a `Response`. `.clicked()` returns `true` on the
frame the mouse button is released over the button.

Other useful `Response` methods:
```rust
response.hovered()   // true while mouse is over the widget
response.double_clicked()
response.drag_delta() // Vec2 — for draggable widgets
```

---

### Full Bayes update method
```rust
// Called once per frame after sampling the sensor:
fn bayes_update(&mut self, mx: f32, my: f32, canvas_w: f32, canvas_h: f32) {
    // 1. Compute likelihood for this reading
    compute_likelihood(&mut self.likelihood, mx, my, self.sigma, canvas_w, canvas_h);

    // 2. Multiply belief × likelihood
    for (b, &l) in self.belief.iter_mut().zip(self.likelihood.iter()) {
        *b *= l;
    }

    // 3. Normalize
    let sum: f32 = self.belief.iter().sum();
    if sum > 1e-10 {
        for v in &mut self.belief { *v /= sum; }
    } else {
        let n = self.belief.len();
        self.belief.fill(1.0 / n as f32);
    }
}
```

---

## What to Implement
1. Add `belief: Vec<f32>` initialized to uniform.
2. Replace the Step 6 single-frame likelihood with a `bayes_update` call each frame.
3. Render the `belief` grid (not `likelihood`) using relative color scaling.
4. Add a "Reset Belief" button in the sidebar.

## Visual Result
- Startup: uniform grey-blue canvas.
- Sensor readings accumulate: hot red region forms around the robot.
- Move robot far away: hot region stays put (no motion handling yet).
- "Reset Belief" button: heatmap goes uniform again.

## Hints
- `belief` replaces the Step 6 rendering, but keep `likelihood` as a scratch buffer
  for the per-frame computation.
- Without relative scaling (`v / max_val`), the heatmap may look completely dark
  after 50+ updates because absolute values become tiny.
