use eframe::egui;

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
    is_initialized: bool, //false by default
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
            }

            let center = egui::pos2(self.robot.x, self.robot.y);
            let radius = 8.0;
            let color = egui::Color32::from_rgb(50, 150, 255);
            let tip =
                center + egui::vec2(20.0 * self.robot.theta.cos(), 20.0 * self.robot.theta.sin());
            painter.line_segment([center, tip], egui::Stroke::new(2.0, egui::Color32::WHITE));
            painter.circle_filled(center, radius, color);
            self.handle_input(ctx, &canvas_rect);

            ui.label("Hi");
        });
    }
}
