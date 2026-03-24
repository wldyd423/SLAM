use eframe::egui;
use rand_distr::{Distribution, Normal};
use std::collections::VecDeque;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Sim", options, Box::new(|_cc| Ok(Box::new(App::default()))))
}

#[derive(Default)]
struct Robot {
    x: f32,
    y: f32,
    theta: f32,
    speed: f32,
    rot_speed: f32,
}

#[derive(Default)]
struct App {
    robot: Robot,
    sigma: f32,
    is_initialized: bool, //false by default
    last_sensor: Option<(f32, f32)>,
    sensor_history: VecDeque<(f32, f32)>,
    sensor_history_limit: usize,
}

impl App {
    fn handle_input(&mut self, ctx: &egui::Context, canvas_rect: &egui::Rect) {
        let (fwd, back, left, right) = ctx.input(|key| {
            (
                key.key_down(egui::Key::ArrowUp),
                key.key_down(egui::Key::ArrowDown),
                key.key_down(egui::Key::ArrowLeft),
                key.key_down(egui::Key::ArrowRight),
            )
        });

        if fwd {
            self.robot.x += self.robot.speed * self.robot.theta.cos();
            self.robot.y += self.robot.speed * self.robot.theta.sin();
        }
        if back {
            self.robot.x -= self.robot.speed * self.robot.theta.cos();
            self.robot.y -= self.robot.speed * self.robot.theta.sin();
        }
        if left {
            self.robot.theta -= self.robot.rot_speed;
        }
        if right {
            self.robot.theta += self.robot.rot_speed;
        }
        self.robot.theta = self.robot.theta.rem_euclid(std::f32::consts::TAU);

        self.robot.x = self.robot.x.clamp(canvas_rect.min.x, canvas_rect.max.x);
        self.robot.y = self.robot.y.clamp(canvas_rect.min.y, canvas_rect.max.y);

        if fwd | back | left | right {
            ctx.request_repaint();
        }
    }
}
// TODO: I was on Step4.md :D
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("control")
            .min_width(180.0)
            .show(ctx, |ui| {
                ui.heading("Controls");
                ui.separator();
                ui.add(
                    egui::Slider::new(&mut self.sigma, 0.0_f32..=100.0)
                        .text("Sensor sigma")
                        .suffix(" px"),
                );
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            let (res, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
            let canvas_rect = res.rect;
            if !self.is_initialized {
                self.robot.x = canvas_rect.center().x;
                self.robot.y = canvas_rect.center().y;
                self.robot.speed = 2.0;
                self.robot.rot_speed = 0.05;
                self.is_initialized = true;
                self.sigma = 20.0;
                self.sensor_history_limit = 100;
            }
            let mut rng = rand::rng();
            let dist = Normal::new(0.0_f32, self.sigma).unwrap();
            let mx = self.robot.x + dist.sample(&mut rng);
            let my = self.robot.y + dist.sample(&mut rng);
            self.last_sensor = Some((mx, my));
            self.sensor_history.push_back((mx, my));
            if self.sensor_history.len() > self.sensor_history_limit {
                self.sensor_history.pop_front();
            }
            for &(x, y) in &self.sensor_history {
                painter.circle_filled(
                    egui::pos2(x, y),
                    3.0,
                    egui::Color32::from_rgba_unmultiplied(255, 80, 80, 80),
                );
            }

            let center = egui::pos2(self.robot.x, self.robot.y);
            let radius = 8.0;
            let color = egui::Color32::from_rgb(50, 150, 255);
            let tip =
                center + egui::vec2(20.0 * self.robot.theta.cos(), 20.0 * self.robot.theta.sin());
            painter.line_segment([center, tip], egui::Stroke::new(2.0, egui::Color32::WHITE));
            painter.circle_filled(center, radius, color);
            self.handle_input(ctx, &canvas_rect);

            //println!("dbg: {:?}", self.last_sensor);
            ui.label("Hi");
            ctx.request_repaint();
        });
    }
}
