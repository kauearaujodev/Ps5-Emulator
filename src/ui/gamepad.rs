// ============================================
// RENDER COM CORES OFICIAIS
// ============================================

impl GamepadScreen {
    pub fn render_face_buttons(&mut self, ui: &mut Ui) {
        let renderer = GamepadRenderer::new(&self.controller_type);
        
        ui.vertical_centered(|ui| {
            // Linha 1: TRIANGLE (Amarelo)
            ui.horizontal(|ui| {
                ui.add_space(40.0);
                let res = renderer.render_triangle(ui, self.state.triangle, "△");
                if res.clicked() { self.state.triangle = !self.state.triangle; }
                ui.add_space(40.0);
            });
            
            ui.add_space(10.0);
            
            // Linha 2: SQUARE (Rosa) + CIRCLE (Vermelho)
            ui.horizontal(|ui| {
                let res = renderer.render_square(ui, self.state.square, "□");
                if res.clicked() { self.state.square = !self.state.square; }
                
                ui.add_space(30.0);
                
                let res = renderer.render_circle(ui, self.state.circle, "○");
                if res.clicked() { self.state.circle = !self.state.circle; }
            });
            
            ui.add_space(10.0);
            
            // Linha 3: CROSS (Azul)
            ui.horizontal(|ui| {
                ui.add_space(40.0);
                let res = renderer.render_cross(ui, self.state.cross, "✕");
                if res.clicked() { self.state.cross = !self.state.cross; }
                ui.add_space(40.0);
            });
        });
    }

    pub fn render_shoulders(&mut self, ui: &mut Ui) {
        let renderer = GamepadRenderer::new(&self.controller_type);
        
        ui.horizontal(|ui| {
            // L2
            self.render_trigger(ui, "L2", &mut self.state.l2, renderer.colors.l2);
            ui.add_space(10.0);
            
            // L1
            self.render_shoulder(ui, "L1", &mut self.state.l1, renderer.colors.l1);
            
            ui.add_space(80.0);
            
            // R1
            self.render_shoulder(ui, "R1", &mut self.state.r1, renderer.colors.r1);
            ui.add_space(10.0);
            
            // R2
            self.render_trigger(ui, "R2", &mut self.state.r2, renderer.colors.r2);
        });
    }

    fn render_shoulder(&self, ui: &mut Ui, label: &str, pressed: &mut bool, color: Color32) {
        let color = if *pressed { color } else { Color32::from_rgb(40, 40, 60) };
        let text_color = if *pressed { Color32::BLACK } else { Color32::WHITE };
        
        let button = Button::new(
            RichText::new(label)
                .size(18.0)
                .color(text_color)
        )
        .fill(color)
        .rounding(8.0)
        .min_size(Vec2::new(60.0, 40.0));
        
        if ui.add(button).clicked() {
            *pressed = !*pressed;
        }
    }

    fn render_trigger(&self, ui: &mut Ui, label: &str, pressed: &mut bool, color: Color32) {
        let color = if *pressed { color } else { Color32::from_rgb(30, 30, 50) };
        let text_color = if *pressed { Color32::BLACK } else { Color32::WHITE };
        
        let button = Button::new(
            RichText::new(format!("{} ▣", label))
                .size(16.0)
                .color(text_color)
        )
        .fill(color)
        .rounding(8.0)
        .min_size(Vec2::new(80.0, 50.0));
        
        if ui.add(button).clicked() {
            *pressed = !*pressed;
        }
    }

    pub fn update_from_mapper(&mut self, mapper: &mut GamepadMapper) {
        mapper.update(&mut self.state);
    }
          }
