use egui::*;
use super::responsive::*;
use super::gamepad::*;

pub struct ResponsiveGamepadScreen {
    pub gamepad: GamepadScreen,
    pub screen_info: ScreenInfo,
}

impl ResponsiveGamepadScreen {
    pub fn new(controller_type: super::controls::ControllerType) -> Self {
        Self {
            gamepad: GamepadScreen::new(controller_type),
            screen_info: ScreenInfo::new(&egui::Context::default()),
        }
    }

    pub fn render(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        self.screen_info = ScreenInfo::new(ctx);
        
        let is_phone = self.screen_info.is_phone();
        let is_portrait = self.screen_info.is_portrait();
        let control_size = self.screen_info.get_control_size();
        let spacing = self.screen_info.get_control_spacing();
        
        // Container do controle
        let frame = Frame::none()
            .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 200))
            .rounding(20.0)
            .inner_margin(if is_phone { 10.0 } else { 20.0 });
        
        frame.show(ui, |ui| {
            if is_portrait {
                self.render_portrait(ui, control_size, spacing);
            } else {
                self.render_landscape(ui, control_size, spacing);
            }
        });
    }

    fn render_portrait(&mut self, ui: &mut Ui, size: f32, spacing: f32) {
        ui.vertical_centered(|ui| {
            // Linha 1: L2, L1, R1, R2
            self.render_shoulders(ui, size * 0.7, spacing);
            
            ui.add_space(spacing);
            
            // Linha 2: D-Pad, Face Buttons
            ui.horizontal(|ui| {
                // D-Pad
                ui.group(|ui| {
                    ui.label(RichText::new("D-PAD").size(12.0).color(Color32::GRAY));
                    self.render_dpad(ui, size * 0.8);
                });
                
                ui.add_space(spacing);
                
                // Face Buttons
                ui.group(|ui| {
                    ui.label(RichText::new("FACE").size(12.0).color(Color32::GRAY));
                    self.render_face_buttons(ui, size * 0.8);
                });
            });
            
            ui.add_space(spacing);
            
            // Linha 3: SELECT, HOME, START
            self.render_center_buttons(ui, size * 0.6, spacing);
            
            ui.add_space(spacing);
            
            // Linha 4: Joysticks
            ui.horizontal(|ui| {
                self.render_joystick(ui, "L", size);
                ui.add_space(spacing * 2.0);
                self.render_joystick(ui, "R", size);
            });
        });
    }

    fn render_landscape(&mut self, ui: &mut Ui, size: f32, spacing: f32) {
        ui.horizontal_centered(|ui| {
            // Lado esquerdo: Joystick L, D-Pad
            ui.vertical(|ui| {
                self.render_joystick(ui, "L", size);
                ui.add_space(spacing);
                self.render_dpad(ui, size * 0.8);
            });
            
            ui.add_space(spacing * 2.0);
            
            // Centro: Shoulders, Face Buttons, Center Buttons
            ui.vertical(|ui| {
                self.render_shoulders(ui, size * 0.6, spacing);
                ui.add_space(spacing);
                self.render_face_buttons(ui, size * 0.9);
                ui.add_space(spacing);
                self.render_center_buttons(ui, size * 0.5, spacing);
            });
            
            ui.add_space(spacing * 2.0);
            
            // Lado direito: Joystick R
            ui.vertical(|ui| {
                self.render_joystick(ui, "R", size);
            });
        });
    }

    fn render_shoulders(&mut self, ui: &mut Ui, size: f32, spacing: f32) {
        ui.horizontal(|ui| {
            self.render_trigger(ui, "L2", &mut self.gamepad.state.l2, size);
            ui.add_space(spacing * 0.5);
            self.render_shoulder(ui, "L1", &mut self.gamepad.state.l1, size);
            
            ui.add_space(spacing * 2.0);
            
            self.render_shoulder(ui, "R1", &mut self.gamepad.state.r1, size);
            ui.add_space(spacing * 0.5);
            self.render_trigger(ui, "R2", &mut self.gamepad.state.r2, size);
        });
    }

    fn render_shoulder(&self, ui: &mut Ui, label: &str, pressed: &mut bool, size: f32) {
        let color = if *pressed {
            Color32::from_rgb(200, 200, 200)
        } else {
            Color32::from_rgb(50, 50, 70)
        };
        
        let btn = Button::new(
            RichText::new(label)
                .size(size * 0.35)
                .color(if *pressed { Color32::BLACK } else { Color32::WHITE })
        )
        .fill(color)
        .rounding(6.0)
        .min_size(Vec2::new(size * 1.2, size * 0.6));
        
        if ui.add(btn).clicked() {
            *pressed = !*pressed;
        }
    }

    fn render_trigger(&self, ui: &mut Ui, label: &str, pressed: &mut bool, size: f32) {
        let color = if *pressed {
            Color32::from_rgb(100, 100, 100)
        } else {
            Color32::from_rgb(30, 30, 50)
        };
        
        let btn = Button::new(
            RichText::new(format!("{} ▣", label))
                .size(size * 0.3)
                .color(if *pressed { Color32::BLACK } else { Color32::WHITE })
        )
        .fill(color)
        .rounding(6.0)
        .min_size(Vec2::new(size * 1.5, size * 0.8));
        
        if ui.add(btn).clicked() {
            *pressed = !*pressed;
        }
    }

    fn render_dpad(&mut self, ui: &mut Ui, size: f32) {
        let state = &mut self.gamepad.state;
        let btn_size = size * 0.3;
        let gap = size * 0.05;
        
        ui.grid("dpad_responsive").show(ui, |ui| {
            // UP
            ui.add_space(btn_size + gap);
            self.render_dpad_btn(ui, "▲", &mut state.up, btn_size);
            ui.add_space(btn_size + gap);
            ui.end_row();
            
            // LEFT + RIGHT
            self.render_dpad_btn(ui, "◄", &mut state.left, btn_size);
            ui.add_space(btn_size + gap * 2.0);
            self.render_dpad_btn(ui, "►", &mut state.right, btn_size);
            ui.end_row();
            
            // DOWN
            ui.add_space(btn_size + gap);
            self.render_dpad_btn(ui, "▼", &mut state.down, btn_size);
            ui.add_space(btn_size + gap);
            ui.end_row();
        });
    }

    fn render_dpad_btn(&self, ui: &mut Ui, label: &str, pressed: &mut bool, size: f32) {
        let color = if *pressed {
            Color32::from_rgb(150, 150, 150)
        } else {
            Color32::from_rgb(50, 50, 70)
        };
        
        let btn = Button::new(
            RichText::new(label)
                .size(size * 0.5)
                .color(if *pressed { Color32::BLACK } else { Color32::WHITE })
        )
        .fill(color)
        .rounding(4.0)
        .min_size(Vec2::new(size, size));
        
        if ui.add(btn).clicked() {
            *pressed = !*pressed;
        }
    }

    fn render_face_buttons(&mut self, ui: &mut Ui, size: f32) {
        let state = &mut self.gamepad.state;
        let btn_size = size * 0.35;
        
        // Cores oficiais
        let triangle_color = Color32::from_rgb(255, 215, 0);
        let circle_color = Color32::from_rgb(255, 0, 0);
        let cross_color = Color32::from_rgb(0, 0, 255);
        let square_color = Color32::from_rgb(255, 105, 180);
        
        ui.vertical_centered(|ui| {
            // TRIANGLE
            ui.horizontal(|ui| {
                ui.add_space(btn_size);
                self.render_face_btn(ui, "△", &mut state.triangle, triangle_color, btn_size);
                ui.add_space(btn_size);
            });
            
            ui.add_space(btn_size * 0.2);
            
            // SQUARE + CIRCLE
            ui.horizontal(|ui| {
                self.render_face_btn(ui, "□", &mut state.square, square_color, btn_size);
                ui.add_space(btn_size * 0.3);
                self.render_face_btn(ui, "○", &mut state.circle, circle_color, btn_size);
            });
            
            ui.add_space(btn_size * 0.2);
            
            // CROSS
            ui.horizontal(|ui| {
                ui.add_space(btn_size);
                self.render_face_btn(ui, "✕", &mut state.cross, cross_color, btn_size);
                ui.add_space(btn_size);
            });
        });
    }

    fn render_face_btn(&self, ui: &mut Ui, label: &str, pressed: &mut bool, color: Color32, size: f32) {
        let bg = if *pressed { color } else { Color32::from_rgb(40, 40, 60) };
        let text_color = if *pressed { Color32::BLACK } else { color };
        
        let btn = Button::new(
            RichText::new(label)
                .size(size * 0.5)
                .color(text_color)
        )
        .fill(bg)
        .rounding(6.0)
        .min_size(Vec2::new(size, size));
        
        if ui.add(btn).clicked() {
            *pressed = !*pressed;
        }
    }

    fn render_center_buttons(&mut self, ui: &mut Ui, size: f32, spacing: f32) {
        ui.horizontal(|ui| {
            self.render_center_btn(ui, "SELECT", &mut self.gamepad.state.select, size);
            ui.add_space(spacing);
            self.render_center_btn(ui, "🏠", &mut self.gamepad.state.home, size);
            ui.add_space(spacing);
            self.render_center_btn(ui, "START", &mut self.gamepad.state.start, size);
        });
    }

    fn render_center_btn(&self, ui: &mut Ui, label: &str, pressed: &mut bool, size: f32) {
        let color = if *pressed {
            Color32::from_rgb(200, 200, 200)
        } else {
            Color32::from_rgb(50, 50, 70)
        };
        
        let btn = Button::new(
            RichText::new(label)
                .size(size * 0.3)
                .color(if *pressed { Color32::BLACK } else { Color32::WHITE })
        )
        .fill(color)
        .rounding(6.0)
        .min_size(Vec2::new(size * 1.5, size * 0.6));
        
        if ui.add(btn).clicked() {
            *pressed = !*pressed;
        }
    }

    fn render_joystick(&mut self, ui: &mut Ui, label: &str, size: f32) {
        let state = &mut self.gamepad.state;
        let (x, y) = if label == "L" {
            (&mut state.left_x, &mut state.left_y)
        } else {
            (&mut state.right_x, &mut state.right_y)
        };
        
        ui.group(|ui| {
            ui.label(RichText::new(label).size(14.0).color(Color32::GRAY));
            
            let joy_size = size * 1.2;
            let center = Vec2::new(joy_size / 2.0, joy_size / 2.0);
            let radius = joy_size / 2.0 - 5.0;
            
            let rect = ui.allocate_space(Vec2::new(joy_size, joy_size));
            let painter = ui.painter();
            
            // Fundo
            painter.circle_filled(
                rect.min + center,
                radius,
                Color32::from_rgb(30, 30, 50)
            );
            painter.circle_stroke(
                rect.min + center,
                radius,
                Stroke::new(2.0, Color32::from_rgb(60, 60, 80))
            );
            
            // Stick
            let stick_x = rect.min.x + center.x + (*x * radius * 0.7);
            let stick_y = rect.min.y + center.y + (*y * radius * 0.7);
            
            painter.circle_filled(
                Pos2::new(stick_x, stick_y),
                radius * 0.3,
                Color32::from_rgb(80, 80, 120)
            );
            painter.circle_stroke(
                Pos2::new(stick_x, stick_y),
                radius * 0.3,
                Stroke::new(2.0, Color32::from_rgb(100, 100, 160))
            );
            
            // Drag
            let response = ui.interact(rect, Id::new(label), Sense::drag());
            if response.dragged() {
                let delta = response.drag_delta();
                *x = (*x + delta.x / radius).clamp(-1.0, 1.0);
                *y = (*y + delta.y / radius).clamp(-1.0, 1.0);
            }
            if response.clicked() {
                *x = 0.0;
                *y = 0.0;
            }
        });
    }
                             }
