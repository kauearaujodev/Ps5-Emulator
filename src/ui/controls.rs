use egui::*;
use super::gamepad::{GamepadMapper, GamepadState};

#[derive(Clone, Debug, PartialEq)]
pub enum ControllerType {
    DualSense,   // PS5
    DualShock,   // PS4
    Xbox,        // Xbox
    Gamepad,     // Genérico
}

pub struct ControlsScreen {
    pub selected: Option<ControllerType>,
    pub gamepad_mapper: GamepadMapper,
    pub show_gamepad_status: bool,
    pub detected_gamepad: String,
}

impl ControlsScreen {
    pub fn new() -> Self {
        let mapper = GamepadMapper::new();
        let detected = mapper.get_gamepad_name();
        
        Self {
            selected: None,
            gamepad_mapper: mapper,
            show_gamepad_status: true,
            detected_gamepad: detected,
        }
    }

    pub fn render(&mut self, ui: &mut Ui) -> Option<ControllerType> {
        let mut result = None;
        
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            // ============================================
            // STATUS DO GAMEPAD CONECTADO
            // ============================================
            if self.show_gamepad_status {
                self.gamepad_mapper.scan_gamepads();
                let count = self.gamepad_mapper.get_connected_count();
                
                ui.horizontal_centered(|ui| {
                    if count > 0 {
                        ui.label(
                            RichText::new("🎮 Gamepad conectado:")
                                .size(16.0)
                                .color(Color32::GREEN)
                        );
                        ui.label(
                            RichText::new(&self.gamepad_mapper.get_gamepad_name())
                                .size(16.0)
                                .color(Color32::WHITE)
                                .strong()
                        );
                    } else {
                        ui.label(
                            RichText::new("🔌 Nenhum gamepad conectado")
                                .size(16.0)
                                .color(Color32::GRAY)
                        );
                    }
                });
                
                ui.add_space(20.0);
            }
            
            // ============================================
            // TÍTULO
            // ============================================
            ui.label(
                RichText::new("🎮 Selecione seu Controle")
                    .size(32.0)
                    .color(Color32::from_rgb(0, 60, 160))
                    .strong()
            );
            
            ui.add_space(40.0);
            
            // ============================================
            // OPÇÕES DE CONTROLE
            // ============================================
            ui.horizontal(|ui| {
                // DualSense (PS5)
                if self.render_control_card(
                    ui,
                    "🎮 DualSense",
                    "PS5",
                    "Controle oficial do PlayStation 5",
                    "Compatible: PC, PS5, Android",
                    ControllerType::DualSense,
                ) {
                    self.selected = Some(ControllerType::DualSense);
                }
                
                ui.add_space(30.0);
                
                // DualShock (PS4)
                if self.render_control_card(
                    ui,
                    "🎮 DualShock",
                    "PS4",
                    "Controle oficial do PlayStation 4",
                    "Compatible: PC, PS4, Android",
                    ControllerType::DualShock,
                ) {
                    self.selected = Some(ControllerType::DualShock);
                }
                
                ui.add_space(30.0);
                
                // Xbox
                if self.render_control_card(
                    ui,
                    "🎮 Xbox",
                    "Xbox",
                    "Controle oficial do Xbox",
                    "Compatible: PC, Xbox, Android",
                    ControllerType::Xbox,
                ) {
                    self.selected = Some(ControllerType::Xbox);
                }
            });
            
            ui.add_space(40.0);
            
            // ============================================
            // BOTÃO CONFIRMAR
            // ============================================
            if self.selected.is_some() {
                let is_gamepad_connected = self.gamepad_mapper.get_connected_count() > 0;
                
                let text = if is_gamepad_connected {
                    "✅ CONFIRMAR (Gamepad detectado)"
                } else {
                    "✅ CONFIRMAR (Touch mode)"
                };
                
                if ui.add(
                    Button::new(
                        RichText::new(text)
                            .size(20.0)
                            .color(Color32::WHITE)
                    )
                    .fill(Color32::from_rgb(0, 200, 80))
                    .rounding(12.0)
                    .min_size(Vec2::new(400.0, 60.0))
                ).clicked() {
                    result = self.selected.clone();
                }
            }
        });
        
        result
    }

    fn render_control_card(
        &mut self,
        ui: &mut Ui,
        name: &str,
        model: &str,
        description: &str,
        compatibility: &str,
        controller_type: ControllerType,
    ) -> bool {
        let is_selected = self.selected == Some(controller_type.clone());
        let is_connected = self.gamepad_mapper.get_connected_count() > 0;
        
        let frame = Frame::none()
            .fill(if is_selected {
                Color32::from_rgba_unmultiplied(0, 60, 160, 100)
            } else {
                Color32::from_rgb(30, 30, 45)
            })
            .rounding(16.0)
            .stroke(Stroke::new(
                if is_selected { 3.0 } else { 1.0 },
                if is_selected {
                    Color32::from_rgb(0, 120, 255)
                } else {
                    Color32::from_rgb(60, 60, 80)
                }
            ))
            .inner_margin(24.0);
        
        let mut clicked = false;
        
        frame.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                // Ícone grande
                ui.label(
                    RichText::new("🎮")
                        .size(64.0)
                );
                
                ui.add_space(10.0);
                
                // Nome
                ui.label(
                    RichText::new(name)
                        .size(22.0)
                        .color(Color32::WHITE)
                        .strong()
                );
                
                // Modelo
                ui.label(
                    RichText::new(model)
                        .size(16.0)
                        .color(Color32::from_rgb(0, 160, 255))
                );
                
                ui.add_space(10.0);
                
                // Descrição
                ui.label(
                    RichText::new(description)
                        .size(14.0)
                        .color(Color32::GRAY)
                );
                
                ui.label(
                    RichText::new(compatibility)
                        .size(12.0)
                        .color(Color32::from_rgb(100, 100, 140))
                );
                
                ui.add_space(10.0);
                
                // Status de conexão
                if is_connected {
                    ui.label(
                        RichText::new("✅ Gamepad detectado")
                            .size(12.0)
                            .color(Color32::GREEN)
                    );
                }
                
                ui.add_space(15.0);
                
                // Botão selecionar
                let button_text = if is_selected { 
                    "✅ SELECIONADO" 
                } else if is_connected && !is_selected {
                    "🔌 CONECTAR"
                } else {
                    "👆 SELECIONAR"
                };
                
                let button_color = if is_selected {
                    Color32::from_rgb(0, 200, 80)
                } else if is_connected && !is_selected {
                    Color32::from_rgb(0, 100, 200)
                } else {
                    Color32::from_rgb(0, 60, 160)
                };
                
                let button = Button::new(
                    RichText::new(button_text)
                        .size(16.0)
                        .color(Color32::WHITE)
                )
                .fill(button_color)
                .rounding(8.0)
                .min_size(Vec2::new(200.0, 45.0));
                
                if ui.add(button).clicked() {
                    clicked = true;
                }
            });
        });
        
        clicked
    }

    pub fn update_gamepad(&mut self) {
        self.gamepad_mapper.scan_gamepads();
        self.detected_gamepad = self.gamepad_mapper.get_gamepad_name();
    }
                  }
