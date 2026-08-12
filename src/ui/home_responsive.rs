use egui::*;
use super::responsive::*;
use super::home::*;

pub struct ResponsiveHomeScreen {
    pub home: HomeScreen,
    pub screen_info: ScreenInfo,
}

impl ResponsiveHomeScreen {
    pub fn new() -> Self {
        Self {
            home: HomeScreen::new(),
            screen_info: ScreenInfo::new(&egui::Context::default()),
        }
    }

    pub fn render(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        // Atualiza informações da tela
        self.screen_info = ScreenInfo::new(ctx);
        
        let padding = self.screen_info.get_padding();
        let font_size = self.screen_info.get_font_size();
        
        // Container principal com padding adaptativo
        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(padding);
                
                // ==========================================
                // HEADER RESPONSIVO
                // ==========================================
                self.render_header(ui, font_size);
                
                ui.add_space(padding * 2.0);
                
                // ==========================================
                // BARRA DE PESQUISA RESPONSIVA
                // ==========================================
                self.render_search(ui, padding);
                
                ui.add_space(padding * 2.0);
                
                // ==========================================
                // GRID DE JOGOS RESPONSIVO
                // ==========================================
                self.render_game_grid(ui);
            });
    }

    fn render_header(&mut self, ui: &mut Ui, font_size: f32) {
        ui.horizontal(|ui| {
            // Logo
            ui.label(
                RichText::new("🎮 PS5")
                    .size(font_size * 1.8)
                    .color(Color32::from_rgb(0, 60, 160))
                    .strong()
            );
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Informações do dispositivo
                let device_icon = match self.screen_info.device {
                    DeviceType::Phone => "📱",
                    DeviceType::Tablet => "📱",
                    DeviceType::Chromebook => "💻",
                    DeviceType::Desktop => "🖥️",
                };
                
                ui.label(
                    RichText::new(format!("{} {}", device_icon, self.home.games.len()))
                        .size(font_size * 0.9)
                        .color(Color32::GRAY)
                );
            });
        });
    }

    fn render_search(&mut self, ui: &mut Ui, padding: f32) {
        let search_width = if self.screen_info.is_phone() {
            ui.available_width() - padding * 2.0
        } else {
            ui.available_width() * 0.6
        };
        
        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut self.home.search_query)
                    .hint_text("🔍 Buscar jogo...")
                    .desired_width(search_width)
                    .font(egui::FontId::proportional(16.0))
            );
            
            let button_text = if self.screen_info.is_phone() { "🔄" } else { "🔄 Atualizar" };
            
            if ui.button(RichText::new(button_text).size(16.0)).clicked() {
                self.home.scan_games();
            }
        });
    }

    fn render_game_grid(&mut self, ui: &mut Ui) {
        let cols = self.screen_info.get_grid_cols();
        let card_size = self.screen_info.get_game_card_size();
        let padding = self.screen_info.get_padding();
        
        let games: Vec<_> = if self.home.search_query.is_empty() {
            self.home.games.clone()
        } else {
            let query = self.home.search_query.to_lowercase();
            self.home.games.iter()
                .filter(|g| g.title.to_lowercase().contains(&query))
                .cloned()
                .collect()
        };
        
        if games.is_empty() {
            self.render_empty_state(ui);
            return;
        }
        
        egui::Grid::new("responsive_games_grid")
            .spacing(Vec2::new(padding, padding))
            .min_col_width(card_size.x)
            .max_col_width(card_size.x + 20.0)
            .show(ui, |ui| {
                for (i, game) in games.iter().enumerate() {
                    self.render_game_card(ui, game, card_size);
                    if (i + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    fn render_game_card(&mut self, ui: &mut Ui, game: &GameInfo, size: Vec2) {
        let is_touch = self.screen_info.is_touch;
        let padding = self.screen_info.get_padding();
        
        let frame = Frame::none()
            .fill(Color32::from_rgb(30, 30, 45))
            .rounding(12.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(0, 60, 160)))
            .inner_margin(padding);
        
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                // Capa
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("🎮")
                            .size(if is_touch { 50.0 } else { 40.0 })
                    );
                });
                
                ui.add_space(5.0);
                
                // Título
                ui.label(
                    RichText::new(&game.title)
                        .size(if is_touch { 14.0 } else { 12.0 })
                        .color(Color32::WHITE)
                        .strong()
                        .wrap()
                );
                
                ui.add_space(2.0);
                
                // ID
                ui.label(
                    RichText::new(&game.title_id)
                        .size(10.0)
                        .color(Color32::GRAY)
                );
                
                ui.add_space(5.0);
                
                // Botão Jogar (touch friendly)
                let btn_height = if is_touch { 45.0 } else { 35.0 };
                let btn_size = Vec2::new(size.x - padding * 2.0, btn_height);
                
                let button = Button::new(
                    RichText::new("▶ JOGAR")
                        .size(if is_touch { 14.0 } else { 12.0 })
                        .color(Color32::WHITE)
                )
                .fill(Color32::from_rgb(0, 120, 255))
                .rounding(8.0)
                .min_size(btn_size);
                
                if ui.add(button).clicked() {
                    self.home.selected_game = Some(self.home.games.iter().position(|g| g.title == game.title).unwrap_or(0));
                    self.home.show_loading = true;
                }
            });
        });
    }

    fn render_empty_state(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            
            ui.label(
                RichText::new("📂 Nenhum jogo encontrado")
                    .size(if self.screen_info.is_phone() { 20.0 } else { 28.0 })
                    .color(Color32::GRAY)
            );
            
            ui.add_space(10.0);
            
            ui.label(
                RichText::new("Coloque seus jogos .pkg na pasta /games/")
                    .size(if self.screen_info.is_phone() { 14.0 } else { 16.0 })
                    .color(Color32::GRAY)
            );
            
            ui.add_space(20.0);
            
            if ui.button("📁 Abrir pasta de jogos").clicked() {
                self.open_games_folder();
            }
        });
    }

    fn open_games_folder(&self) {
        let path = &self.home.games_path;
        
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("explorer").arg(path).spawn();
        
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
          }
