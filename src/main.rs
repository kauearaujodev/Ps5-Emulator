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
use std::path::PathBuf;
use eframe::egui;
use rayon::prelude::*;
use parking_lot::Mutex as FastMutex;

// ============================================
// SISTEMA DE ARQUIVOS
// ============================================
mod fs {
    use std::fs;
    use std::path::{Path, PathBuf};
    use serde::{Deserialize, Serialize};
    use dirs;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct AppConfig {
        pub games_folder: PathBuf,
        pub saves_folder: PathBuf,
        pub audio_volume: f32,
        pub music_volume: f32,
        pub sfx_volume: f32,
        pub language: String,
        pub quality: String,
        pub last_game: Option<String>,
    }

    impl Default for AppConfig {
        fn default() -> Self {
            let home = dirs::home_dir().unwrap_or(PathBuf::from("."));
            Self {
                games_folder: home.join("PS5Emulator/games"),
                saves_folder: home.join("PS5Emulator/saves"),
                audio_volume: 1.0,
                music_volume: 0.8,
                sfx_volume: 0.9,
                language: "pt-BR".to_string(),
                quality: "high".to_string(),
                last_game: None,
            }
        }
    }

    pub struct FileSystem {
        pub config: AppConfig,
        config_path: PathBuf,
    }

    impl FileSystem {
        pub fn new() -> Self {
            let home = dirs::home_dir().unwrap_or(PathBuf::from("."));
            let config_path = home.join("PS5Emulator/config.json");
            
            let config = if config_path.exists() {
                Self::load_config(&config_path).unwrap_or_default()
            } else {
                AppConfig::default()
            };

            // Cria pastas necessárias
            Self::create_directories(&config);

            Self {
                config,
                config_path,
            }
        }

        fn load_config(path: &Path) -> Result<AppConfig, String> {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Erro ao ler config: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("Erro ao parse config: {}", e))
        }

        fn save_config(&self) -> Result<(), String> {
            let content = serde_json::to_string_pretty(&self.config)
                .map_err(|e| format!("Erro ao serializar config: {}", e))?;
            fs::write(&self.config_path, content)
                .map_err(|e| format!("Erro ao salvar config: {}", e))
        }

        fn create_directories(config: &AppConfig) {
            let _ = fs::create_dir_all(&config.games_folder);
            let _ = fs::create_dir_all(&config.saves_folder);
        }

        pub fn save_game_data(&self, game_id: &str, data: &[u8]) -> Result<(), String> {
            let save_path = self.config.saves_folder.join(format!("{}.save", game_id));
            fs::write(save_path, data)
                .map_err(|e| format!("Erro ao salvar jogo: {}", e))
        }

        pub fn load_game_data(&self, game_id: &str) -> Result<Vec<u8>, String> {
            let save_path = self.config.saves_folder.join(format!("{}.save", game_id));
            fs::read(save_path)
                .map_err(|e| format!("Erro ao carregar jogo: {}", e))
        }

        pub fn list_games(&self) -> Vec<String> {
            let mut games = Vec::new();
            if let Ok(entries) = fs::read_dir(&self.config.games_folder) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "pkg").unwrap_or(false) {
                        if let Some(name) = path.file_stem() {
                            games.push(name.to_string_lossy().to_string());
                        }
                    }
                }
            }
            games
        }

        pub fn update_config(&mut self, config: AppConfig) -> Result<(), String> {
            self.config = config;
            self.save_config()
        }
    }
}

// ============================================
// RENDERIZAÇÃO DA CPU (GPU)
// ============================================
mod render {
    use wgpu::*;
    use std::sync::Arc;

    pub struct GpuRenderer {
        pub surface: wgpu::Surface,
        pub device: wgpu::Device,
        pub queue: wgpu::Queue,
        pub config: wgpu::SurfaceConfiguration,
        pub size: winit::dpi::PhysicalSize<u32>,
        pub initialized: bool,
    }

    impl GpuRenderer {
        pub async fn new(window: Arc<winit::window::Window>) -> Self {
            let size = window.inner_size();
            
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: Backends::all(),
                dx12_shader_compiler: Default::default(),
            });

            let surface = unsafe { instance.create_surface(&*window).unwrap() };

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap();

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
                initialized: true,
            }
        }

        pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
            if !self.initialized {
                return Ok(());
            }

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
    cpu: Arc<FastMutex<Ps5Cpu>>,
    memory: Arc<FastMutex<VirtualMemory>>,
    library: Arc<FastMutex<GameLibrary>>,
    
    // Áudio
    audio: Arc<AudioEngine>,
    audio_controller: AudioController,
    
    // GPU
    renderer: Option<render::GpuRenderer>,
    
    // Sistema de Arquivos
    fs: Arc<FastMutex<fs::FileSystem>>,
    
    // Interface
    home: ResponsiveHomeScreen,
    loading: Option<LoadingScreen>,
    controls: ControlsScreen,
    gamepad: Option<ResponsiveGamepadScreen>,
    
    // Estado
    current_screen: Screen,
    game_loaded: bool,
    game_name: String,
    game_id: String,
    frame_count: u64,
    last_frame_time: std::time::Instant,
    fps: f32,
    cpu_time: f32,
    render_time: f32,
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
        let cpu_arc = Arc::new(FastMutex::new(cpu));
        let memory_arc = Arc::new(FastMutex::new(memory));
        
        let library = Arc::new(FastMutex::new(GameLibrary::new(
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
        // INICIALIZA SISTEMA DE ARQUIVOS
        // ============================================
        let fs = Arc::new(FastMutex::new(fs::FileSystem::new()));
        
        // ============================================
        // INICIALIZA INTERFACE
        // ============================================
        let mut home = ResponsiveHomeScreen::new();
        
        // Carrega jogos do sistema de arquivos
        {
            let fs_lock = fs.lock();
            let games = fs_lock.list_games();
            // Adiciona jogos encontrados à biblioteca
            for game_name in games {
                // Simula adição de jogo
                let game = GamePackage::new(
                    format!("CUSA-{}", game_name.len() + 10000),
                    game_name.clone(),
                    "1.0.0".to_string()
                );
                // Adiciona à biblioteca (simplificado)
            }
        }
        home.home.scan_games();
        
        Self {
            cpu: cpu_arc,
            memory: memory_arc,
            library,
            audio: audio_arc,
            audio_controller,
            renderer: None,
            fs,
            home,
            loading: None,
            controls: ControlsScreen::new(),
            gamepad: None,
            current_screen: Screen::Home,
            game_loaded: false,
            game_name: String::new(),
            game_id: String::new(),
            frame_count: 0,
            last_frame_time: std::time::Instant::now(),
            fps: 0.0,
            cpu_time: 0.0,
            render_time: 0.0,
        }
    }

    // ============================================
    // EXECUTA UM PASSO DA CPU (OTIMIZADO)
    // ============================================
    fn cpu_step(&mut self) -> Result<(), String> {
        let start = std::time::Instant::now();
        
        let mut cpu = self.cpu.lock();
        let mut memory = self.memory.lock();
        
        // Executa um passo em cada core (paralelizado)
        let cores: Vec<_> = (0..8).collect();
        
        // Usa Rayon para paralelizar os cores
        cores.par_iter().for_each(|&core_id| {
            let _ = cpu.step_core(core_id, &mut memory);
        });
        
        self.cpu_time = start.elapsed().as_secs_f32();
        
        Ok(())
    }

    // ============================================
    // RODA O JOGO
    // ============================================
    fn run_game(&mut self, title_id: &str, profile: &str) -> Result<(), String> {
        // 1. Carrega o jogo
        let library = self.library.lock();
        let game_info = library.get_game_info(title_id)
            .ok_or("Jogo não encontrado")?;
        
        if !game_info.is_installed {
            return Err("Jogo não está instalado".to_string());
        }
        
        self.game_id = title_id.to_string();
        self.game_name = game_info.title_name.clone();
        drop(library);
        
        // 2. Inicia o jogo na CPU
        let mut cpu = self.cpu.lock();
        let mut memory = self.memory.lock();
        
        // Carrega código do jogo
        let game_data = vec![
            0x48, 0xB8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x48, 0xB9, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x01, 0xC8,
            0xF4,
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
        
        // 4. Carrega save
        let fs_lock = self.fs.lock();
        if let Ok(save_data) = fs_lock.load_game_data(title_id) {
            println!("💾 Save carregado: {} bytes", save_data.len());
        }
        drop(fs_lock);
        
        // 5. Marca como carregado
        self.game_loaded = true;
        
        // 6. Salva último jogo na config
        {
            let mut fs_lock = self.fs.lock();
            fs_lock.config.last_game = Some(title_id.to_string());
            let _ = fs_lock.save_config();
        }
        
        Ok(())
    }

    // ============================================
    // SALVA O JOGO
    // ============================================
    fn save_game(&self) -> Result<(), String> {
        if !self.game_loaded {
            return Err("Nenhum jogo carregado".to_string());
        }
        
        // Coleta estado do jogo
        let cpu = self.cpu.lock();
        let memory = self.memory.lock();
        
        // Cria dados do save
        let save_data = format!(
            "game={};pc={};registers={:?}",
            self.game_name,
            cpu.cores[0].rip,
            cpu.cores[0].registers
        );
        
        drop(cpu);
        drop(memory);
        
        // Salva no sistema de arquivos
        let fs_lock = self.fs.lock();
        fs_lock.save_game_data(&self.game_id, save_data.as_bytes())?;
        drop(fs_lock);
        
        println!("💾 Jogo salvo: {}", self.game_name);
        
        // Toca som de save
        let test_tone = audio::loader::AudioLoader::generate_test_tone(880, 0.1, 48000);
        let _ = self.audio.play_sound("save", test_tone);
        
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
            let render_start = std::time::Instant::now();
            if let Some(renderer) = &mut self.renderer {
                let _ = renderer.render();
            }
            self.render_time = render_start.elapsed().as_secs_f32();
        }
        
        // ============================================
        // INTERFACE PRINCIPAL
        // ============================================
        egui::CentralPanel::default().show(ctx, |ui| {
            // Layout: conteúdo principal + painel de áudio
            ui.horizontal(|ui| {
                // Conteúdo principal (75%)
                ui.vertical(|ui| {
                    match self.current_screen {
                        Screen::Home => {
                            self.home.render(ui, ctx);
                            
                            if self.home.home.show_loading {
                                if let Some(game) = self.home.home.selected_game {
                                    self.game_name = self.home.home.games[game].title.clone();
                                    self.game_id = self.home.home.games[game].title_id.clone();
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
                                // Inicializa GPU
                                // (winit/wgpu em app separado)
                                self.gamepad = Some(ResponsiveGamepadScreen::new(controller));
                                self.current_screen = Screen::Game;
                                self.game_loaded = true;
                                
                                // Toca som de jogo iniciado
                                let test_tone = audio::loader::AudioLoader::generate_test_tone(880, 0.2, 48000);
                                let _ = self.audio.play_sound("game_start", test_tone);
                            }
                        }
                        
                        Screen::Game => {
                            // ============================================
                            // RODA A CPU
                            // ============================================
                            let cpu_start = std::time::Instant::now();
                            let _ = self.cpu_step();
                            let cpu_time = cpu_start.elapsed().as_secs_f32() * 1000.0;
                            
                            // ============================================
                            // INTERFACE DO JOGO
                            // ============================================
                            ui.vertical(|ui| {
                                // Top bar
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("🎮 {}", self.game_name))
                                        .size(24.0)
                                        .color(Color32::GREEN));
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(format!("FPS: {:.0}", self.fps))
                                            .size(14.0)
                                            .color(Color32::GRAY));
                                        
                                        ui.label(RichText::new(format!("CPU: {:.1}ms", cpu_time))
                                            .size(14.0)
                                            .color(Color32::GRAY));
                                    });
                                });
                                
                                ui.add_space(10.0);
                                
                                // Status
                                ui.horizontal(|ui| {
                                    let cpu = self.cpu.lock();
                                    let total_instructions = cpu.total_instructions;
                                    let active_cores = cpu.cores.iter()
                                        .filter(|c| !c.halted)
                                        .count();
                                    drop(cpu);
                                    
                                    ui.label(RichText::new(format!("⚡ Cores: {}/8", active_cores))
                                        .size(14.0)
                                        .color(Color32::GRAY));
                                    
                                    ui.label(RichText::new(format!("📊 Instruções: {}", total_instructions))
                                        .size(14.0)
                                        .color(Color32::GRAY));
                                    
                                    // Botão Save
                                    if ui.button(RichText::new("💾 SAVE").size(16.0)).clicked() {
                                        let _ = self.save_game();
                                    }
                                });
                                
                                ui.add_space(20.0);
                                
                                // ============================================
                                // CONTROLES
                                // ============================================
                                if let Some(gamepad) = &mut self.gamepad {
                                    self.controls.update_gamepad();
                                    gamepad.gamepad.update_from_mapper(&mut self.controls.gamepad_mapper);
                                    gamepad.render(ui, ctx);
                                }
                            });
                        }
                    }
                });

                // ============================================
                // PAINEL DE ÁUDIO (25%)
                // ============================================
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    
                    // Título
                    ui.label(RichText::new("🔊 ÁUDIO")
                        .size(18.0)
                        .color(Color32::from_rgb(0, 160, 255))
                        .strong());
                    
                    ui.add_space(10.0);
                    
                    // Controles de áudio
                    self.audio_controller.render(ui);
                    
                    ui.add_space(20.0);
                    
                    // ============================================
                    // SISTEMA DE ARQUIVOS - Configurações
                    // ============================================
                    ui.collapsing("📁 Configurações", |ui| {
                        let mut fs_lock = self.fs.lock();
                        
                        ui.label(RichText::new("Pasta de jogos:")
                            .size(14.0)
                            .color(Color32::GRAY));
                        ui.label(RichText::new(
                            fs_lock.config.games_folder.to_string_lossy()
                        ).size(12.0).color(Color32::GRAY));
                        
                        ui.add_space(5.0);
                        
                        ui.label(RichText::new("Pasta de saves:")
                            .size(14.0)
                            .color(Color32::GRAY));
                        ui.label(RichText::new(
                            fs_lock.config.saves_folder.to_string_lossy()
                        ).size(12.0).color(Color32::GRAY));
                        
                        ui.add_space(10.0);
                        
                        // Quality
                        ui.horizontal(|ui| {
                            ui.label("Qualidade:");
                            ui.selectable_value(&mut fs_lock.config.quality, "low".to_string(), "Baixa");
                            ui.selectable_value(&mut fs_lock.config.quality, "medium".to_string(), "Média");
                            ui.selectable_value(&mut fs_lock.config.quality, "high".to_string(), "Alta");
                        });
                        
                        if ui.button("💾 Salvar Configurações").clicked() {
                            let _ = fs_lock.save_config();
                            println!("✅ Configurações salvas!");
                        }
                        
                        drop(fs_lock);
                    });
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
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([360.0, 600.0])
            .with_max_inner_size([3840.0, 2160.0])
            .with_title("🎮 PS5 Emulator - Ultimate Edition"),
        ..Default::default()
    };

    eframe::run_native(
        "PS5 Emulator",
        options,
        Box::new(|_cc| Box::new(Ps5EmulatorApp::new())),
    )
                    }
