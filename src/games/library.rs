use super::package::*;
use super::installer::*;
use crate::cpu::Ps5Cpu;
use crate::memory::VirtualMemory;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct GameLibraryEntry {
    pub title_id: String,
    pub title_name: String,
    pub package_id: String,
    pub version: String,
    pub is_installed: bool,
    pub is_updated: bool,
    pub last_played: u64,
    pub play_time: u64,
    pub progress: f32,
    pub achievements: Vec<Achievement>,
    pub rating: u8,
    pub reviews: Vec<GameReview>,
}

#[derive(Clone, Debug)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked: bool,
    pub unlock_time: Option<u64>,
    pub points: u32,
    pub rarity: f32,
}

#[derive(Clone, Debug)]
pub struct GameReview {
    pub user_id: String,
    pub rating: u8,
    pub comment: String,
    pub timestamp: u64,
    pub helpful: u32,
}

#[derive(Clone, Debug)]
pub struct GameSession {
    pub package_id: String,
    pub start_time: u64,
    pub last_save: Option<u64>,
    pub total_time: u64,
    pub current_level: String,
    pub player_health: f32,
    pub player_position: (f32, f32, f32),
    pub game_state: HashMap<String, String>,
}

pub struct GameLibrary {
    pub library: HashMap<String, GameLibraryEntry>,
    pub installer: Arc<Mutex<Installer>>,
    pub active_session: Option<GameSession>,
    pub cpu: Arc<Mutex<Ps5Cpu>>,
    pub memory: Arc<Mutex<VirtualMemory>>,
}

impl GameLibrary {
    pub fn new(
        cpu: Arc<Mutex<Ps5Cpu>>,
        memory: Arc<Mutex<VirtualMemory>>,
    ) -> Self {
        let installer = Arc::new(Mutex::new(Installer::new(
            memory.lock().unwrap().clone()
        )));

        Self {
            library: HashMap::new(),
            installer,
            active_session: None,
            cpu,
            memory,
        }
    }

    pub fn add_game(
        &mut self,
        package: GamePackage,
        cpu: &mut Ps5Cpu,
        memory: &mut VirtualMemory,
    ) -> Result<String, String> {
        let package_id = package.package_id.clone();
        let title_id = package.title_id.clone();
        let title_name = package.title_name.clone();
        let version = package.version.clone();

        // Registra o pacote no installer
        let install_id = self.installer.lock().unwrap()
            .register_package(package);

        // Adiciona à biblioteca
        let entry = GameLibraryEntry {
            title_id: title_id.clone(),
            title_name: title_name.clone(),
            package_id: install_id.clone(),
            version: version.clone(),
            is_installed: false,
            is_updated: false,
            last_played: 0,
            play_time: 0,
            progress: 0.0,
            achievements: Vec::new(),
            rating: 0,
            reviews: Vec::new(),
        };

        self.library.insert(title_id.clone(), entry);

        Ok(install_id)
    }

    pub fn install_game(
        &mut self,
        title_id: &str,
        profile_id: &str,
    ) -> Result<(), String> {
        let entry = self.library
            .get_mut(title_id)
            .ok_or("Game not found in library")?;

        if entry.is_installed {
            return Err("Game already installed".to_string());
        }

        // Instala o pacote
        self.installer.lock().unwrap()
            .install_package(&entry.package_id, profile_id)?;

        entry.is_installed = true;

        // Carrega o jogo na CPU
        let game_data = self.installer.lock().unwrap()
            .get_installation(&entry.package_id)
            .map(|inst| {
                // Simula carregamento do jogo
                let mut data = Vec::new();
                for i in 0..10 {
                    data.push((i & 0xFF) as u8);
                }
                data
            })
            .unwrap_or_else(Vec::new);

        let mut cpu = self.cpu.lock().unwrap();
        let mut memory = self.memory.lock().unwrap();

        // Carrega o jogo na memória virtual
        cpu.load_game(&mut memory, &game_data)?;

        Ok(())
    }

    pub fn launch_game(
        &mut self,
        title_id: &str,
        profile_id: &str,
    ) -> Result<(), String> {
        let entry = self.library
            .get_mut(title_id)
            .ok_or("Game not found")?;

        if !entry.is_installed {
            return Err("Game not installed".to_string());
        }

        // Cria sessão de jogo
        let session = GameSession {
            package_id: entry.package_id.clone(),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_save: None,
            total_time: entry.play_time,
            current_level: "Level 1".to_string(),
            player_health: 100.0,
            player_position: (0.0, 0.0, 0.0),
            game_state: HashMap::new(),
        };

        self.active_session = Some(session);

        // Executa o jogo na CPU
        let mut cpu = self.cpu.lock().unwrap();
        let mut memory = self.memory.lock().unwrap();

        // Simula execução do jogo
        cpu.run(&mut memory, 1000)?;

        entry.last_played = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    pub fn save_game_progress(&mut self) -> Result<(), String> {
        let session = self.active_session
            .as_mut()
            .ok_or("No active game session")?;

        let package_id = &session.package_id;
        
        // Cria save data
        let mut save_data = GameSaveData::new(
            package_id.clone(),
            "profile_001".to_string()
        );
        
        // Salva estado do jogo
        let state = format!(
            "level={};health={};pos={},{},{};time={}",
            session.current_level,
            session.player_health,
            session.player_position.0,
            session.player_position.1,
            session.player_position.2,
            session.total_time
        );
        
        save_data.set_data(state.into_bytes());
        save_data.add_metadata("platform".to_string(), "PS5".to_string());

        // Salva no installer
        self.installer.lock().unwrap()
            .save_game(package_id, "profile_001", save_data)?;

        session.last_save = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());

        Ok(())
    }

    pub fn load_game_progress(&mut self) -> Result<(), String> {
        let session = self.active_session
            .as_mut()
            .ok_or("No active game session")?;

        let package_id = &session.package_id;

        // Carrega save data
        let save_data = self.installer.lock().unwrap()
            .load_save(package_id, "profile_001")?;

        // Parse do estado salvo
        let state_str = String::from_utf8(save_data)
            .map_err(|e| format!("Invalid save data: {}", e))?;

        for part in state_str.split(';') {
            let parts: Vec<&str> = part.split('=').collect();
            if parts.len() == 2 {
                match parts[0] {
                    "level" => session.current_level = parts[1].to_string(),
                    "health" => {
                        if let Ok(health) = parts[1].parse::<f32>() {
                            session.player_health = health;
                        }
                    }
                    "pos" => {
                        let pos: Vec<&str> = parts[1].split(',').collect();
                        if pos.len() == 3 {
                            if let (Ok(x), Ok(y), Ok(z)) = (
                                pos[0].parse::<f32>(),
                                pos[1].parse::<f32>(),
                                pos[2].parse::<f32>(),
                            ) {
                                session.player_position = (x, y, z);
                            }
                        }
                    }
                    "time" => {
                        if let Ok(time) = parts[1].parse::<u64>() {
                            session.total_time = time;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub fn get_game_info(&self, title_id: &str) -> Option<&GameLibraryEntry> {
        self.library.get(title_id)
    }

    pub fn get_all_games(&self) -> Vec<&GameLibraryEntry> {
        self.library.values().collect()
    }

    pub fn get_installed_games(&self) -> Vec<&GameLibraryEntry> {
        self.library.values()
            .filter(|g| g.is_installed)
            .collect()
    }

    pub fn search_games(&self, query: &str) -> Vec<&GameLibraryEntry> {
        let query_lower = query.to_lowercase();
        self.library.values()
            .filter(|g| {
                g.title_name.to_lowercase().contains(&query_lower) ||
                g.title_id.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    pub fn unlock_achievement(&mut self, achievement_id: &str) -> Result<(), String> {
        let session = self.active_session
            .as_mut()
            .ok_or("No active game session")?;

        let entry = self.library
            .get_mut(&session.package_id)
            .ok_or("Game not found")?;

        if let Some(achievement) = entry.achievements
            .iter_mut()
            .find(|a| a.id == achievement_id) {
            
            if !achievement.unlocked {
                achievement.unlocked = true;
                achievement.unlock_time = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );
                println!("🏆 Achievement unlocked: {}", achievement.name);
            }
        }

        Ok(())
    }

    pub fn add_achievement(&mut self, title_id: &str, achievement: Achievement) -> Result<(), String> {
        let entry = self.library
            .get_mut(title_id)
            .ok_or("Game not found")?;

        entry.achievements.push(achievement);
        Ok(())
    }

    pub fn get_game_stats(&self, title_id: &str) -> Option<GameStats> {
        self.library.get(title_id).map(|entry| {
            let total_achievements = entry.achievements.len();
            let unlocked_achievements = entry.achievements
                .iter()
                .filter(|a| a.unlocked)
                .count();

            GameStats {
                title_name: entry.title_name.clone(),
                version: entry.version.clone(),
                is_installed: entry.is_installed,
                play_time: entry.play_time,
                last_played: entry.last_played,
                progress: entry.progress,
                achievements_total: total_achievements as u32,
                achievements_unlocked: unlocked_achievements as u32,
                rating: entry.rating,
                total_reviews: entry.reviews.len() as u32,
            }
        })
    }
}

pub struct GameStats {
    pub title_name: String,
    pub version: String,
    pub is_installed: bool,
    pub play_time: u64,
    pub last_played: u64,
    pub progress: f32,
    pub achievements_total: u32,
    pub achievements_unlocked: u32,
    pub rating: u8,
    pub total_reviews: u32,
}

// Exemplo de jogo demo para testar
pub fn create_demo_game() -> GamePackage {
    let mut package = GamePackage::new(
        "CUSA-12345".to_string(),
        "Demon Slayer: Virtual Edition".to_string(),
        "1.0.0".to_string()
    );

    // Adiciona dados do jogo
    let game_code = vec![
        0x48, 0xB8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // MOV RAX, 1
        0x48, 0xB9, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // MOV RCX, 2
        0x48, 0x01, 0xC8,                                             // ADD RAX, RCX
        0xF4,                                                          // HALT
    ];

    package.add_chunk(game_code, false);

    // Adiciona ícone (simplificado)
    package.icon_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    // Adiciona metadados
    package.metadata.insert("developer".to_string(), "Virtual Studios".to_string());
    package.metadata.insert("publisher".to_string(), "Sony Interactive".to_string());
    package.metadata.insert("genre".to_string(), "Action RPG".to_string());
    package.metadata.insert("rating".to_string(), "M".to_string());

    package
          }
