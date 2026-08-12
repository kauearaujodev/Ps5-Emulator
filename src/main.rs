mod memory;
mod cpu;
mod games;
mod ui;

use memory::VirtualMemory;
use cpu::Ps5Cpu;
use games::prelude::*;
use ui::*;
use std::sync::{Arc, Mutex};
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([360.0, 600.0])
            .with_title("🎮 PS5 Emulator"),
        ..Default::default()
    };

    eframe::run_native(
        "PS5 Emulator",
        options,
        Box::new(|_cc| Box::new(Ps5EmulatorApp::new())),
    )
}

struct Ps5EmulatorApp {
    cpu: Arc<Mutex<Ps5Cpu>>,
    memory: Arc<Mutex<VirtualMemory>>,
    library: Arc<Mutex<GameLibrary>>,
    home: ResponsiveHomeScreen,
    loading: Option<LoadingScreen>,
    controls: ControlsScreen,
    gamepad: Option<ResponsiveGamepadScreen>,
    current_screen: Screen,
    game_loaded: bool,
    game_name: String,
}

#[derive(PartialEq)]
enum Screen {
    Home,
    Loading,
    Controls,
    Game,
}

impl Ps5EmulatorApp {
    pub fn new() -> Self {
        let memory = VirtualMemory::new(2 * 1024 * 1024 * 1024);
        let cpu = Ps5Cpu::new();
        let cpu_arc = Arc::new(Mutex::new(cpu));
        let memory_arc = Arc::new(Mutex::new(memory));
        
        let library = Arc::new(Mutex::new(GameLibrary::new(
            cpu_arc.clone(), 
            memory_arc.clone()
        )));
        
        Self {
            cpu: cpu_arc,
            memory: memory_arc,
            library,
            home: ResponsiveHomeScreen::new(),
            loading: None,
            controls: ControlsScreen::new(),
            gamepad: None,
            current_screen: Screen::Home,
            game_loaded: false,
            game_name: String::new(),
        }
    }
}

impl eframe::App for Ps5EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configuração adaptativa
        let screen_info = ScreenInfo::new(ctx);
        let scale = if screen_info.is_phone() { 1.5 } else { 1.0 };
        ctx.set_pixels_per_point(scale);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_screen {
                Screen::Home => {
                    self.home.render(ui, ctx);
                    
                    if self.home.home.show_loading {
                        if let Some(game) = self.home.home.selected_game {
                            self.game_name = self.home.home.games[game].title.clone();
                            self.loading = Some(LoadingScreen::new(self.game_name.clone()));
                            self.current_screen = Screen::Loading;
                            self.home.home.show_loading = false;
                        }
                    }
                }
                
                Screen::Loading => {
                    if let Some(loading) = &mut self.loading {
                        let finished = loading.render(ui);
                        if finished {
                            self.current_screen = Screen::Controls;
                        }
                    }
                }
                
                Screen::Controls => {
                    if let Some(controller) = self.controls.render(ui) {
                        self.gamepad = Some(ResponsiveGamepadScreen::new(controller));
                        self.current_screen = Screen::Game;
                        self.game_loaded = true;
                    }
                }
                
                Screen::Game => {
                    if let Some(gamepad) = &mut self.gamepad {
                        self.controls.update_gamepad();
                        gamepad.gamepad.update_from_mapper(&mut self.controls.gamepad_mapper);
                        gamepad.render(ui, ctx);
                    }
                }
            }
        });
        
        ctx.request_repaint();
    }
                      }
