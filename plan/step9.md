# Step 9 — Particle Filter Alternative

## SLAM Concept
The grid filter works but scales poorly: a 100×100 grid in 2-D becomes 100³ in 3-D
or 100⁶ for a full 6-DOF pose. **Particle filters** solve this by representing the
belief as a set of N weighted samples (particles) drawn from the distribution.

Three steps per cycle:
1. **Propagate** — move each particle by the same motion command plus random noise
2. **Weight** — score each particle by how well it explains the sensor reading
3. **Resample** — draw N new particles from the current weighted set (likely
   particles survive; unlikely ones die)

This is **Sequential Importance Resampling (SIR)** — the standard particle filter.

---

## Library Reference: structs as data, weighted iteration, egui toggles

### Defining `Particle` and the filter
```rust
// A particle: a hypothesis about where the robot is, plus its probability weight.
#[derive(Clone)]  // needed for resampling (we copy surviving particles)
struct Particle {
    x: f32,
    y: f32,
    w: f32,  // weight — proportional to how likely this position is
}

struct ParticleFilter {
    particles: Vec<Particle>,
    motion_sigma: f32,   // noise added during propagate
}
```

---

### Initializing particles uniformly over the canvas
```rust
use rand::Rng;

fn init_particles(n: usize, canvas_w: f32, canvas_h: f32) -> Vec<Particle> {
    let mut rng = rand::rng();
    (0..n).map(|_| Particle {
        x: rng.random_range(0.0..canvas_w),
        y: rng.random_range(0.0..canvas_h),
        w: 1.0 / n as f32,
    }).collect()
    // rng.random_range(lo..hi) — uniform random f32 in [lo, hi)
}
```

---

### Propagate — add motion + noise to every particle
```rust
use rand_distr::{Distribution, Normal};

fn propagate(particles: &mut Vec<Particle>, dx: f32, dy: f32, sigma: f32) {
    let noise = Normal::new(0.0_f32, sigma).unwrap();
    let mut rng = rand::rng();
    for p in particles.iter_mut() {
        // Move by the same delta as the robot, plus independent random noise:
        p.x += dx + noise.sample(&mut rng);
        p.y += dy + noise.sample(&mut rng);
        // Clamp to canvas (replace with real bounds):
        p.x = p.x.clamp(0.0, 800.0);
        p.y = p.y.clamp(0.0, 600.0);
    }
}
// dx, dy = how far the robot moved this frame (from your motion model)
```

`iter_mut()` gives a mutable reference to each element — you can modify `p.x`,
`p.y`, `p.w` in place. It's equivalent to `for p in &mut particles`.

---

### Weight — score each particle against the sensor reading
```rust
fn update_weights(particles: &mut Vec<Particle>, mx: f32, my: f32, sigma: f32) {
    let two_sigma_sq = 2.0 * sigma * sigma;

    // 1. Compute raw Gaussian likelihood for each particle:
    for p in particles.iter_mut() {
        let dist_sq = (p.x - mx).powi(2) + (p.y - my).powi(2);
        p.w = (- dist_sq / two_sigma_sq).exp();
        // Particles far from (mx,my) → w near 0
        // Particles near (mx,my)     → w near 1
    }

    // 2. Normalize so weights sum to 1:
    let total: f32 = particles.iter().map(|p| p.w).sum();
    if total > 1e-10 {
        for p in particles.iter_mut() { p.w /= total; }
    } else {
        // All weights collapsed — reinitialize uniformly:
        let n = particles.len();
        for p in particles.iter_mut() { p.w = 1.0 / n as f32; }
    }
}
```

---

### Systematic resampling — draw N new particles proportional to weights
This is the key step that kills off low-weight particles and duplicates high-weight ones:
```rust
use rand::Rng;

fn resample(particles: &[Particle]) -> Vec<Particle> {
    let n = particles.len();
    let mut rng = rand::rng();

    // Systematic resampling: evenly spaced samples with a single random offset.
    let step = 1.0_f32 / n as f32;
    let start: f32 = rng.random_range(0.0..step);  // one random number for all n draws

    let mut out = Vec::with_capacity(n);
    let mut cumsum = 0.0_f32;
    let mut j = 0;  // index into the old particle array

    for i in 0..n {
        let target = start + i as f32 * step;
        // Advance j until cumulative weight reaches target:
        while j < particles.len() - 1 && cumsum + particles[j].w < target {
            cumsum += particles[j].w;
            j += 1;
        }
        // Particle j is selected — copy it with equal weight:
        out.push(Particle { x: particles[j].x, y: particles[j].y, w: step });
    }
    out
}
// After resampling: all particles have equal weight (step = 1/n).
// Particles from dense regions appear multiple times; sparse particles vanish.
```

---

### Drawing particles — size or alpha by weight
```rust
// Simple: all particles same size, semi-transparent
for p in &self.particles {
    painter.circle_filled(
        egui::pos2(p.x, p.y),
        2.5,
        egui::Color32::from_rgba_unmultiplied(255, 180, 0, 150),  // orange
    );
}

// Advanced: scale radius by weight for visual emphasis
let max_w = particles.iter().map(|p| p.w).fold(0.0_f32, f32::max);
for p in &self.particles {
    let r = 1.5 + 4.0 * (p.w / max_w.max(1e-10));  // radius 1.5–5.5 px
    painter.circle_filled(egui::pos2(p.x, p.y), r, egui::Color32::from_rgb(255, 180, 0));
}
```

---

### Mode toggle with `egui::ComboBox`
```rust
// Define an enum for the current filter mode:
#[derive(PartialEq)]
enum FilterMode { Grid, Particle }

// In the SidePanel:
egui::ComboBox::from_label("Filter mode")
    .selected_text(match self.mode {
        FilterMode::Grid     => "Grid",
        FilterMode::Particle => "Particle",
    })
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut self.mode, FilterMode::Grid,     "Grid");
        ui.selectable_value(&mut self.mode, FilterMode::Particle, "Particle");
    });
```
`ui.selectable_value(&mut var, value, label)` — sets `var = value` when clicked.
`#[derive(PartialEq)]` is required so egui can compare the current value to each option.

Alternatively, use radio buttons:
```rust
ui.radio_value(&mut self.mode, FilterMode::Grid,     "Grid");
ui.radio_value(&mut self.mode, FilterMode::Particle, "Particle");
// radio_value works identically to selectable_value but renders as a radio button.
```

---

### Particle count slider
```rust
ui.add(egui::Slider::new(&mut self.n_particles, 50_usize..=2000).text("Particles"));
// When n_particles changes, reinitialize the particle Vec:
if self.particles.len() != self.n_particles {
    self.particles = init_particles(self.n_particles, canvas_w, canvas_h);
}
```

---

## What to Implement
1. `struct Particle { x, y, w }` + `ParticleFilter` (or fields directly on `SlamApp`).
2. `init_particles` — uniform random spread over canvas.
3. `propagate` — move particles by robot delta + Gaussian noise.
4. `update_weights` — Gaussian likelihood from sensor reading, then normalize.
5. `resample` — systematic resampling.
6. Draw particles (size/alpha by weight optional).
7. Mode toggle in sidebar (`ComboBox` or `radio_value`).

## Visual Result
- 500 dots scattered uniformly at start.
- Move robot: dots diffuse outward (propagate noise).
- Sensor reading: dots cluster near the measurement.
- After resampling: sparse areas disappear, dense cluster tracks the robot.

## Hints
- Start with N = 200; add a slider for 50–2000.
- Resample every frame (or when effective sample size `1/Σw²` drops below N/2)
  to prevent weight degeneracy.
- Put particle logic in `src/filter/particle.rs` for a clean architecture.
- `motion_sigma` for particles should be a separate slider from sensor `σ`.
