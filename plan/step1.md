# Step 1 — Blank Window

## Status: ✅ Complete

## SLAM Concept
Before any estimation, we need a simulation environment. This blank window is the
empty stage — every subsequent step adds a SLAM component on top of it.

---

## Library Reference: eframe / egui basics

### What is eframe vs egui?
- **egui** — the UI framework: widgets, layout, drawing. You call egui to build UI.
- **eframe** — the native desktop wrapper around egui. It owns the OS window, the
  event loop, and calls your `App::update` every frame.

You almost never call eframe functions directly after startup — eframe calls *you*.

---

### `Cargo.toml` dependencies
```toml
[dependencies]
eframe = "0.31"
egui = "0.31"   # usually pulled in transitively, but explicit is fine
```

---

### `eframe::run_native` — open the window and start the loop
```rust
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0]),  // starting window size in logical pixels
        ..Default::default()
    };

    eframe::run_native(
        "SLAM Sim",          // window title bar text
        options,             // window / renderer config
        Box::new(|_cc| Ok(Box::new(SlamApp::default()))),
        // ^^^ a factory closure: eframe calls this once to create your App.
        // `_cc` is CreationContext — gives you access to egui ctx, storage, wgpu
        // device etc. You can ignore it for now.
    )
}
```
`run_native` **blocks** — it runs the event loop until the window is closed, then
returns `Ok(())`.

---

### `eframe::App` trait — the one method you must implement
```rust
impl eframe::App for SlamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Called once per frame (typically 60 fps, vsync-locked).
        // `ctx`   — the egui context: input state, painting, memory.
        // `frame` — the eframe frame: lets you close the window, set window title, etc.
        //           You won't need it until much later.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello SLAM!");
        });
    }
}
```

---

### `egui::CentralPanel` — fills whatever space is left
```rust
egui::CentralPanel::default().show(ctx, |ui| {
    // `ui` is a &mut egui::Ui — the region inside the panel.
    // Everything you add here is laid out top-to-bottom by default.
});
```
`CentralPanel` is special: it must be added last (after any side/top/bottom panels)
and it expands to fill all remaining space. There can only be one per frame.

---

### Minimal working `main.rs`
```rust
use eframe::egui;

struct SlamApp;

impl eframe::App for SlamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_ui| {});
        // Empty body — just keep the window alive.
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "SLAM Sim",
        options,
        Box::new(|_cc| Ok(Box::new(SlamApp))),
    )
}
```

---

## What to Implement
1. `Cargo.toml` dependencies: `eframe`, `egui`
2. `struct SlamApp {}` (empty for now)
3. `impl eframe::App for SlamApp` with an empty `update`
4. `fn main()` calling `eframe::run_native`

## Visual Result
A gray/dark window opens. Nothing drawn yet. Window can be resized and closed.

## Hints
- `eframe::run_native` returns `Result` — propagate it with `?` or `.unwrap()`.
- Default `NativeOptions::default()` is fine; use `ViewportBuilder::with_inner_size`
  if you want a specific starting size.
- The `update` function is called every frame even with an empty body — the window
  stays live and responsive.
