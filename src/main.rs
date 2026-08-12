// ============================================
// MÓDULOS
// ============================================
mod memory;
mod cpu;
mod games;
mod audio;
mod ui;

// ============================================
// IMPORTS
// ============================================
use memory::VirtualMemory;
use cpu::Ps5Cpu;
use games::prelude::*;
use audio::*;
use ui::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use eframe::egui;

// ============================================
// RENDERIZAÇÃO DA CPU (GPU)
// ============================================
mod render {
    use wgpu::*;
    use winit::window::Window;
    use std::sync::Arc;

    pub struct GpuRenderer {
        pub surface: wgpu::Surface,
        pub device: wgpu::Device,
        pub queue: wgpu::Queue,
        pub config: wgpu::SurfaceConfiguration,
        pub size: winit::dpi::PhysicalSize<u32>,
        pub window: Arc<Window>,
    }

    impl GpuRenderer {
        pub async fn new(window: Arc<Window>) -> Self {
            let size = window.inner_size();
            
            // Cria instância wgpu
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: Backends::all(),
                dx12_shader_compiler: Default::default(),
            });

            // Cria surface
            let surface = unsafe { instance.create_surface(&*window).unwrap() };

            // Adaptador
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap();

            // Dispositivo
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .unwrap();

            // Configuração da surface
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface.get_capabilities(&adapter).formats[0],
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            };

            surface.configure(&device, &config);

            Self {
                surface,
                device,
                queue,
                config,
                size,
                window,
            }
        }

        pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
            let output = self.surface.get_current_texture()?;
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.2,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
            }

            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();

            Ok(())
        }
    }
}

// ============================================
// APP PRINCIPAL
// ============================================
struct Ps5EmulatorApp {
    // Hardware
    cpu: Arc<Mutex<Ps5Cpu>>,
    memory: Arc<Mutex<VirtualMemory>>,
    library: Arc<Mutex<GameLibrary>>,
    
    // Áudio
    audio: Arc<AudioEngine>,
    audio_controller: AudioController,
    
    // GPU
    renderer: Option<render::GpuRenderer>,
    
    // Interface
    home: ResponsiveHomeScreen,
    loading: Option<LoadingScreen>,
    controls: ControlsScreen,
    gamepad: Option<ResponsiveGamepadScreen>,
    
    // Estado
    current_screen: Screen,
    game_loaded: bool,
    game_name: String,
    frame_count: u64,
    last_frame_time: std::time::Instant,
    fps: f32,
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
        // ============================================
        // INICIALIZA HARDWARE
        // ============================================
        let memory = VirtualMemory::new(2 * 1024 * 1024 * 1024);
        let cpu = Ps5Cpu::new();
        let cpu_arc = Arc::new(Mutex::new(cpu));
        let memory_arc = Arc::new(Mutex::new(memory));
        
        let library = Arc::new(Mutex::new(GameLibrary::new(
            cpu_arc.clone(), 
            memory_arc.clone()
        )));

        // ============================================
        // INICIALIZA ÁUDIO
        // ============================================
        let mut audio_engine = AudioEngine::new();
        let _ = audio_engine.init();
        let audio_arc = Arc::new(audio_engine);
        let audio_controller = AudioController::new(audio_arc.clone());
        
        // ============================================
        // INICIALIZA INTERFACE
        // ============================================
        let mut home = ResponsiveHomeScreen::new();
        home.home.scan_games();
        
        Self {
            cpu: cpu_arc,
            memory: memory_arc,
            library,
            audio: audio_arc,
            audio_controller,
            renderer: None,
            home,
            loading: None,
            controls: ControlsScreen::new(),
            gamepad: None,
            current_screen: Screen::Home,
            game_loaded: false,
            game_name: String::new(),
            frame_count: 0,
            last_frame_time: std::time::Instant::now(),
            fps: 0.0,
        }
    }

    // ============================================
    // EXECUTA UM PASSO DA CPU
    // ============================================
    fn cpu_step(&mut self) -> Result<(), String> {
        let mut cpu = self.cpu.lock().unwrap();
        let mut memory = self.memory.lock().unwrap();
        
        // Executa um passo em cada core
        for core_id in 0..8 {
            cpu.step_core(core_id, &mut memory)?;
        }
        
        Ok(())
    }

    // ============================================
    // RODA O JOGO
    // ============================================
    fn run_game(&mut self, title_id: &str, profile: &str) -> Result<(), String> {
        // 1. Carrega o jogo
        let library = self.library.lock().unwrap();
        let game_info = library.get_game_info(title_id)
            .ok_or("Jogo não encontrado")?;
        
        if !game_info.is_installed {
            return Err("Jogo não está instalado".to_string());
        }
        
        drop(library);
        
        // 2. Inicia o jogo na CPU
        let mut cpu = self.cpu.lock().unwrap();
        let mut memory = self.memory.lock().unwrap();
        
        // Carrega o jogo na memória
        // (simulação - pega dados do jogo)
        let game_data = vec![
            0x48, 0xB8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // MOV RAX, 1
            0x48, 0xB9, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // MOV RCX, 2
            0x48, 0x01, 0xC8,                                             // ADD RAX, RCX
            0xF4,                                                          // HALT
        ];
        
        for (i, &byte) in game_data.iter().enumerate() {
            memory.write8(0x1000 + i as u64, byte)?;
        }
        
        cpu.cores[0].rip = 0x1000;
        cpu.cores[0].status = cpu::CoreStatus::Active;
        
        drop(cpu);
        drop(memory);
        
        // 3. Toca som de inicialização
        let test_tone = audio::loader::AudioLoader::generate_test_tone(440, 0.3, 48000);
        let _ = self.audio.play_sound("startup", test_tone);
        
        // 4. Marca como carregado
        self.game_loaded = true;
        self.game_name = game_info.title_name.clone();
        
        Ok(())
    }

    // ============================================
    // ATUALIZA FPS
    // ============================================
    fn update_fps(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_frame_time.elapsed();
        
        if elapsed >= Duration::from_secs(1) {
            self.fps = self.frame_count as f32 / elapsed.as_secs_f32();
            self.frame_count = 0;
            self.last_frame_time = std::time::Instant::now();
        }
    }
}

// ============================================
// IMPLEMENTAÇÃO DO eframe::App
// ============================================
impl eframe::App for Ps5EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Atualiza FPS
        self.update_fps();
        
        // Informações da tela
        let screen_info = ScreenInfo::new(ctx);
        let scale = if screen_info.is_phone() { 1.5 } else { 1.0 };
        ctx.set_pixels_per_point(scale);
        
        // ============================================
        // RENDERIZAÇÃO DA GPU (se estiver no jogo)
        // ============================================
        if self.current_screen == Screen::Game {
            if let Some(renderer) = &mut self.renderer {
                let _ = renderer.render();
            }
        }
        
        // ============================================
        // INTERFACE PRINCIPAL
        // ============================================
        egui::CentralPanel::default().show(ctx, |ui| {
            // Layout: conteúdo principal + painel de áudio
            ui.horizontal(|ui| {
                // Conteúdo principal (80%)
                ui.vertical(|ui| {
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
                                    // Inicia o jogo
                                    if let Some(game) = self.home.home.selected_game {
                                        let title_id = &self.home.home.games[game].title_id;
                                        let _ = self.run_game(title_id, "player_001");
                                    }
                                    self.current_screen = Screen::Controls;
                                }
                            }
                        }
                        
                        Screen::Controls => {
                            if let Some(controller) = self.controls.render(ui) {
                                // Inicializa GPU renderer
                                // Nota: isso seria feito com winit/wgpu
                                self.gamepad = Some(ResponsiveGamepadScreen::new(controller));
                                self.current_screen = Screen::Game;
                                self.game_loaded = true;
                                
                                // Toca som de jogo iniciado
                                let test_tone = audio::loader::AudioLoader::generate_test_tone(880, 0.2, 48000);
                                let _ = self.audio.play_sound("game_start", test_tone);
                            }
                        }
                        
                        Screen::Game => {
                            // Executa um passo da CPU
                            let _ = self.cpu_step();
                            
                            // Renderiza o gamepad
                            if let Some(gamepad) = &mut self.gamepad {
                                self.controls.update_gamepad();
                                gamepad.gamepad.update_from_mapper(&mut self.controls.gamepad_mapper);
                                gamepad.render(ui, ctx);
                            }
                            
                            // Mostra FPS e status
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("🎮 {} - Rodando", self.game_name))
                                    .size(20.0)
                                    .color(Color32::GREEN));
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("FPS: {:.0}", self.fps))
                                        .size(14.0)
                                        .color(Color32::GRAY));
                                    
                                    ui.label(RichText::new(format!("🔄 {} frames", self.frame_count))
                                        .size(14.0)
                                        .color(Color32::GRAY));
                                });
                            });
                            
                            // Status da CPU
                            ui.horizontal(|ui| {
                                let cpu = self.cpu.lock().unwrap();
                                let total_instructions = cpu.total_instructions;
                                let active_cores = cpu.cores.iter()
                                    .filter(|c| !c.halted)
                                    .count();
                                drop(cpu);
                                
                                ui.label(RichText::new(format!("⚡ Cores ativos: {}", active_cores))
                                    .size(14.0)
                                    .color(Color32::GRAY));
                                
                                ui.label(RichText::new(format!("📊 Instruções: {}", total_instructions))
                                    .size(14.0)
                                    .color(Color32::GRAY));
                            });
                        }
                    }
                });

                // ============================================
                // PAINEL DE ÁUDIO (20%)
                // ============================================
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    self.audio_controller.render(ui);
                });
            });
        });
        
        // ============================================
        // REPAINT
        // ============================================
        ctx.request_repaint();
    }
}

// ============================================
// MAIN
// ============================================
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
