//! PS5 Virtual Emulator - Ponto de entrada principal
//! 
//! Este arquivo inicializa o emulador, cria jogos demo,
//! e demonstra todas as funcionalidades do sistema.

mod memory;
mod cpu;
mod games;

use memory::VirtualMemory;
use cpu::Ps5Cpu;
use games::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), String> {
    // Configura logging
    env_logger::init();
    
    println!("🎮 PS5 VIRTUAL EMULATOR");
    println!("=====================================\n");
    
    // =============================================
    // 1. INICIALIZAÇÃO DO HARDWARE
    // =============================================
    println!("📀 Inicializando hardware...");
    
    let mut memory = VirtualMemory::new(2 * 1024 * 1024 * 1024); // 2GB
    let mut cpu = Ps5Cpu::new();
    
    println!("✅ CPU: 8 cores Zen 2 @ 3.5GHz");
    println!("✅ Memória: 2GB RAM virtual");
    println!("✅ GPU: RDNA 2 (simulada)");
    println!("✅ SSD: 825GB (simulado)\n");
    
    // =============================================
    // 2. CONFIGURAÇÃO DA BIBLIOTECA
    // =============================================
    println!("📚 Configurando biblioteca de jogos...");
    
    let cpu_arc = Arc::new(Mutex::new(cpu));
    let memory_arc = Arc::new(Mutex::new(memory));
    
    let mut library = GameLibrary::new(cpu_arc.clone(), memory_arc.clone());
    println!("✅ Biblioteca pronta!\n");
    
    // =============================================
    // 3. CRIAÇÃO DOS JOGOS DEMO
    // =============================================
    println!("🎮 Criando jogos demo...\n");
    
    // Jogo 1: Aventura
    let game1 = create_adventure_game();
    let id1 = game1.title_id.clone();
    library.add_game(
        game1,
        &mut cpu_arc.lock().unwrap(),
        &mut memory_arc.lock().unwrap()
    )?;
    println!("✅ Jogo 1: Aventura Espacial ({})", id1);
    
    // Jogo 2: Corrida
    let game2 = create_racing_game();
    let id2 = game2.title_id.clone();
    library.add_game(
        game2,
        &mut cpu_arc.lock().unwrap(),
        &mut memory_arc.lock().unwrap()
    )?;
    println!("✅ Jogo 2: Turbo Racing ({})", id2);
    
    // Jogo 3: Puzzle
    let game3 = create_puzzle_game();
    let id3 = game3.title_id.clone();
    library.add_game(
        game3,
        &mut cpu_arc.lock().unwrap(),
        &mut memory_arc.lock().unwrap()
    )?;
    println!("✅ Jogo 3: Brain Puzzle ({})\n", id3);
    
    // =============================================
    // 4. INSTALAÇÃO DOS JOGOS
    // =============================================
    println!("📥 Instalando jogos...\n");
    
    let profile = "player_001";
    let games = vec![
        (id1.clone(), "Aventura Espacial"),
        (id2.clone(), "Turbo Racing"),
        (id3.clone(), "Brain Puzzle"),
    ];
    
    for (id, name) in &games {
        println!("📦 Instalando {}...", name);
        
        match library.install_game(id, profile) {
            Ok(_) => {
                println!("✅ {} instalado com sucesso!", name);
                
                // Verifica integridade
                if let Some(entry) = library.get_game_info(id) {
                    if let Some(install) = library.installer.lock().unwrap()
                        .get_installation(&entry.package_id) {
                        println!("   📊 Tamanho: {} MB", install.install_size / 1024 / 1024);
                        println!("   📁 Path: {}", install.install_path);
                    }
                }
            }
            Err(e) => println!("❌ Erro ao instalar {}: {}", name, e),
        }
        
        thread::sleep(Duration::from_millis(500));
    }
    println!();
    
    // =============================================
    // 5. ACHIEVEMENTS
    // =============================================
    println!("🏆 Configurando achievements...\n");
    
    // Achievements Aventura
    setup_adventure_achievements(&mut library, &id1)?;
    
    // Achievements Corrida
    setup_racing_achievements(&mut library, &id2)?;
    
    // Achievements Puzzle
    setup_puzzle_achievements(&mut library, &id3)?;
    
    println!("✅ Achievements configurados!\n");
    
    // =============================================
    // 6. EXECUÇÃO DOS JOGOS
    // =============================================
    println!("🎮 INICIANDO SESSÃO DE GAMING");
    println!("=========================================\n");
    
    // ----- JOGO 1: Aventura -----
    println!("🚀 JOGO 1: Aventura Espacial");
    println!("-----------------------------------------\n");
    
    library.launch_game(&id1, profile)?;
    println!("🎯 Jogo carregado!");
    
    // Simula gameplay
    for level in 1..=3 {
        println!("   Nível {}: Explorando o espaço...", level);
        thread::sleep(Duration::from_millis(300));
        
        match level {
            1 => {
                library.unlock_achievement("ACH_ADV_001")?;
                println!("   🏆 Achievement: Primeiro Contato!");
            }
            2 => {
                library.unlock_achievement("ACH_ADV_002")?;
                println!("   🏆 Achievement: Explorador Intergaláctico!");
            }
            3 => {
                library.unlock_achievement("ACH_ADV_003")?;
                println!("   🏆 Achievement: Senhor das Galáxias!");
            }
            _ => {}
        }
    }
    
    library.save_game_progress()?;
    println!("💾 Progresso salvo!\n");
    
    // ----- JOGO 2: Corrida -----
    println!("🏎️ JOGO 2: Turbo Racing");
    println!("-----------------------------------------\n");
    
    library.launch_game(&id2, profile)?;
    println!("🏁 Jogo carregado!");
    
    let tracks = ["Speedway", "City Circuit", "Mountain Pass", "Coastal Road"];
    for (i, track) in tracks.iter().enumerate() {
        println!("   Corrida {}: {}", i + 1, track);
        thread::sleep(Duration::from_millis(200));
        
        if i == 0 {
            library.unlock_achievement("ACH_RAC_001")?;
            println!("   🏆 Achievement: Primeira Vitória!");
        }
        if i == 2 {
            library.unlock_achievement("ACH_RAC_002")?;
            println!("   🏆 Achievement: Rei da Montanha!");
        }
    }
    
    library.save_game_progress()?;
    println!("💾 Progresso salvo!\n");
    
    // ----- JOGO 3: Puzzle -----
    println!("🧩 JOGO 3: Brain Puzzle");
    println!("-----------------------------------------\n");
    
    library.launch_game(&id3, profile)?;
    println!("🧠 Jogo carregado!");
    
    for puzzle in 1..=4 {
        println!("   Puzzle #{}: Resolvendo...", puzzle);
        thread::sleep(Duration::from_millis(300));
        
        match puzzle {
            2 => {
                library.unlock_achievement("ACH_PUZ_001")?;
                println!("   🏆 Achievement: Mestre do Puzzle!");
            }
            4 => {
                library.unlock_achievement("ACH_PUZ_002")?;
                println!("   🏆 Achievement: Gênio!");
            }
            _ => {}
        }
    }
    
    library.save_game_progress()?;
    println!("💾 Progresso salvo!\n");
    
    // =============================================
    // 7. ESTATÍSTICAS
    // =============================================
    println!("=========================================");
    println!("📊 ESTATÍSTICAS DA SESSÃO");
    println!("=========================================\n");
    
    // Estatísticas dos jogos
    println!("🎮 JOGOS INSTALADOS:");
    for (id, name) in &games {
        if let Some(stats) = library.get_game_stats(id) {
            println!("\n📌 {}", name);
            println!("   Versão: {}", stats.version);
            println!("   Instalado: {}", if stats.is_installed { "✅" } else { "❌" });
            println!("   Tempo jogado: {} segundos", stats.play_time);
            println!("   Progresso: {:.1}%", stats.progress);
            println!("   Achievements: {}/{}", 
                stats.achievements_unlocked,
                stats.achievements_total
            );
            if stats.rating > 0 {
                println!("   Avaliação: {}/10 ⭐", stats.rating);
            }
        }
    }
    println!();
    
    // Estatísticas da CPU
    let cpu_stats = cpu_arc.lock().unwrap().get_stats();
    println!("{}", cpu_stats);
    
    // =============================================
    // 8. LIMPEZA
    // =============================================
    println!("\n🗑️ Desinstalando jogos...\n");
    
    for (id, name) in &games {
        if let Some(entry) = library.get_game_info(id) {
            println!("   Removendo {}...", name);
            let _ = library.installer.lock().unwrap()
                .uninstall_package(&entry.package_id);
            println!("   ✅ {} removido!", name);
        }
    }
    
    println!("\n✅ Todos os jogos desinstalados!");
    println!("\n🎮 PS5 Virtual Emulator - Encerrado!");
    println!("=========================================\n");
    
    Ok(())
}

// =============================================
// FUNÇÕES AUXILIARES PARA CRIAR JOGOS
// =============================================

/// Cria um jogo de aventura espacial
fn create_adventure_game() -> GamePackage {
    let mut game = GamePackage::new(
        "CUSA-ADV01".to_string(),
        "Aventura Espacial".to_string(),
        "1.0.0".to_string()
    );
    
    // Código do jogo (simulação)
    let game_code = vec![
        0x48, 0xB8, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x48, 0xB9, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x48, 0x83, 0xE8, 0x01,
        0x48, 0x83, 0xF8, 0x00,
        0x74, 0x03,
        0xEB, 0xF4,
        0xF4,
    ];
    
    game.add_chunk(game_code, false);
    
    game.metadata.insert("developer".to_string(), "Space Studios".to_string());
    game.metadata.insert("genre".to_string(), "Action Adventure".to_string());
    game.metadata.insert("rating".to_string(), "T".to_string());
    
    game
}

/// Cria um jogo de corrida
fn create_racing_game() -> GamePackage {
    let mut game = GamePackage::new(
        "CUSA-RAC01".to_string(),
        "Turbo Racing".to_string(),
        "1.0.0".to_string()
    );
    
    let game_code = vec![
        0x48, 0xB8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x48, 0xB9, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x48, 0x01, 0xC8,
        0x48, 0x3D, 0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x7C, 0xF4,
        0xF4,
    ];
    
    game.add_chunk(game_code, false);
    
    game.metadata.insert("developer".to_string(), "Speed Games".to_string());
    game.metadata.insert("genre".to_string(), "Racing".to_string());
    game.metadata.insert("rating".to_string(), "E".to_string());
    
    game
}

/// Cria um jogo de puzzle
fn create_puzzle_game() -> GamePackage {
    let mut game = GamePackage::new(
        "CUSA-PUZ01".to_string(),
        "Brain Puzzle".to_string(),
        "1.0.0".to_string()
    );
    
    let game_code = vec![
        0x48, 0xB8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x48, 0xB9, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x48, 0xBA, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x48, 0x01, 0xC8,
        0x48, 0x01, 0xD0,
        0xF4,
    ];
    
    game.add_chunk(game_code, false);
    
    game.metadata.insert("developer".to_string(), "Mind Games".to_string());
    game.metadata.insert("genre".to_string(), "Puzzle".to_string());
    game.metadata.insert("rating".to_string(), "E".to_string());
    
    game
}

// =============================================
// ACHIEVEMENTS
// =============================================

fn setup_adventure_achievements(library: &mut GameLibrary, title_id: &str) -> Result<(), String> {
    library.add_achievement(title_id, Achievement {
        id: "ACH_ADV_001".to_string(),
        name: "Primeiro Contato".to_string(),
        description: "Encontre sua primeira civilização alienígena".to_string(),
        unlocked: false,
        unlock_time: None,
        points: 50,
        rarity: 75.0,
    })?;
    
    library.add_achievement(title_id, Achievement {
        id: "ACH_ADV_002".to_string(),
        name: "Explorador Intergaláctico".to_string(),
        description: "Explore 5 sistemas estelares diferentes".to_string(),
        unlocked: false,
        unlock_time: None,
        points: 100,
        rarity: 40.0,
    })?;
    
    library.add_achievement(title_id, Achievement {
        id: "ACH_ADV_003".to_string(),
        name: "Senhor das Galáxias".to_string(),
        description: "Complete o jogo com 100% de progresso".to_string(),
        unlocked: false,
        unlock_time: None,
        points: 200,
        rarity: 15.0,
    })?;
    
    Ok(())
}

fn setup_racing_achievements(library: &mut GameLibrary, title_id: &str) -> Result<(), String> {
    library.add_achievement(title_id, Achievement {
        id: "ACH_RAC_001".to_string(),
        name: "Primeira Vitória".to_string(),
        description: "Vença sua primeira corrida".to_string(),
        unlocked: false,
        unlock_time: None,
        points: 30,
        rarity: 80.0,
    })?;
    
    library.add_achievement(title_id, Achievement {
        id: "ACH_RAC_002".to_string(),
        name: "Rei da Montanha".to_string(),
        description: "Complete a pista da montanha em menos de 2 minutos".to_string(),
        unlocked: false,
        unlock_time: None,
        points: 75,
        rarity: 45.0,
    })?;
    
    Ok(())
}

fn setup_puzzle_achievements(library: &mut GameLibrary, title_id: &str) -> Result<(), String> {
    library.add_achievement(title_id, Achievement {
        id: "ACH_PUZ_001".to_string(),
        name: "Mestre do Puzzle".to_string(),
        description: "Resolva 10 puzzles consecutivos sem erros".to_string(),
        unlocked: false,
        unlock_time: None,
        points: 60,
        rarity: 55.0,
    })?;
    
    library.add_achievement(title_id, Achievement {
        id: "ACH_PUZ_002".to_string(),
        name: "Gênio".to_string(),
        description: "Resolva todos os puzzles do jogo".to_string(),
        unlocked: false,
        unlock_time: None,
        points: 150,
        rarity: 20.0,
    })?;
    
    Ok(())
      }
