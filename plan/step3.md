# Step 3 — Keyboard Control

## SLAM Concept
Motion commands drive the robot. At this stage actuation is **perfect** — no noise,
no slippage. The robot goes exactly where commanded. In later steps we will add motion
noise (the "predict" uncertainty), but first we need a robot we can steer.

This is the **motion model**: `x' = x + v·cos(θ)`, `y' = y + v·sin(θ)`.

---

## Library Reference: input and animation

### `ctx.input(|i| ...)` — reading the keyboard
egui's input system is polled, not event-driven. Each frame you call `ctx.input`
with a closure that receives a snapshot of the current input state:

```rust
ctx.input(|i| {
    // `i` is &egui::InputState — snapshot of keyboard, mouse, touch, scroll, etc.
    // Reading it here is safe; it does NOT mutate any state.
    if i.key_down(egui::Key::ArrowUp) {
        // key is currently held down (fires every frame while held)
    }
    if i.key_pressed(egui::Key::ArrowUp) {
        // key was pressed THIS frame only (fires once per physical key press)
    }
});
```

**`key_down` vs `key_pressed`:**
| Method | When it fires | Use for |
|--------|--------------|---------|
| `key_down(k)` | Every frame while key is held | Smooth continuous motion |
| `key_pressed(k)` | Once when key transitions off→on | Single-shot actions (reset, toggle) |

---

### Reading multiple keys in one closure
```rust
// Collect what we need from input, then mutate self after the closure.
let (fwd, back, left, right) = ctx.input(|i| (
    i.key_down(egui::Key::ArrowUp),
    i.key_down(egui::Key::ArrowDown),
    i.key_down(egui::Key::ArrowLeft),
    i.key_down(egui::Key::ArrowRight),
));

// Now mutate outside the closure (borrow checker is happy):
if fwd  { self.robot.x += SPEED * self.robot.theta.cos();
          self.robot.y += SPEED * self.robot.theta.sin(); }
if back { self.robot.x -= SPEED * self.robot.theta.cos();
          self.robot.y -= SPEED * self.robot.theta.sin(); }
if left  { self.robot.theta -= ROT_SPEED; }
if right { self.robot.theta += ROT_SPEED; }
```
The closure borrows `ctx` immutably, so you cannot mutate `self` inside it.
Collect flags as owned booleans, then act on them outside.

---

### `ctx.request_repaint()` — keep the animation running
By default egui only redraws when it receives input (mouse move, key press, etc.).
If the robot should keep moving while a key is held, call this to force a repaint
next frame:
```rust
// At the end of update():
ctx.request_repaint();
// Now update() is called every frame, not just on events.
```
Without this, the robot may appear to stutter when held keys are between OS
key-repeat intervals.

---

### `f32::clamp` — keep the robot on screen
After computing new coordinates, clamp them to the canvas bounds:
```rust
// canvas_rect: egui::Rect — the drawable area (from allocate_painter)
self.robot.x = self.robot.x.clamp(canvas_rect.min.x, canvas_rect.max.x);
self.robot.y = self.robot.y.clamp(canvas_rect.min.y, canvas_rect.max.y);
```
`egui::Rect` has `.min` (top-left `Pos2`) and `.max` (bottom-right `Pos2`).
Until you have a real `canvas_rect`, use numeric constants like `0.0..=800.0`.

---

### Keeping `theta` in range
Without clamping, `theta` grows without bound. Use `rem_euclid` to wrap it to
`[0, 2π)`:
```rust
use std::f32::consts::TAU; // TAU = 2π
self.robot.theta = self.robot.theta.rem_euclid(TAU);
```
This is equivalent to `theta % TAU` but always returns a positive value.

---

### Full motion update sketch
```rust
const SPEED: f32 = 2.0;        // pixels per frame
const ROT_SPEED: f32 = 0.05;   // radians per frame

fn handle_input(&mut self, ctx: &egui::Context) {
    let (fwd, back, left, right) = ctx.input(|i| (
        i.key_down(egui::Key::ArrowUp),
        i.key_down(egui::Key::ArrowDown),
        i.key_down(egui::Key::ArrowLeft),
        i.key_down(egui::Key::ArrowRight),
    ));

    if fwd {
        self.robot.x += SPEED * self.robot.theta.cos();
        self.robot.y += SPEED * self.robot.theta.sin();
    }
    if back {
        self.robot.x -= SPEED * self.robot.theta.cos();
        self.robot.y -= SPEED * self.robot.theta.sin();
    }
    if left  { self.robot.theta -= ROT_SPEED; }
    if right { self.robot.theta += ROT_SPEED; }

    self.robot.theta = self.robot.theta.rem_euclid(std::f32::consts::TAU);

    // Clamp to canvas (replace with real rect later)
    self.robot.x = self.robot.x.clamp(0.0, 800.0);
    self.robot.y = self.robot.y.clamp(0.0, 600.0);
}
```

---

## What to Implement
1. Add `const SPEED` and `const ROT_SPEED` constants.
2. In `update`, read the four arrow keys and mutate `self.robot`.
3. Clamp robot position to canvas bounds.
4. Call `ctx.request_repaint()` at the end of `update`.

## Visual Result
Arrow keys steer the robot smoothly around the canvas. The heading arrow rotates
visibly. The robot cannot leave the canvas area.

## Hints
- Put the input handling in a dedicated method `fn handle_input(&mut self, ctx)` to
  keep `update` readable.
- `egui::Key` has variants for most keys: `ArrowUp`, `ArrowDown`, `ArrowLeft`,
  `ArrowRight`, `Space`, `Escape`, `A`–`Z`, etc.
