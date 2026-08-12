use super::package::*;
use crate::memory::VirtualMemory;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum InstallStatus {
    NotInstalled,
    Installing { progress: f32, current_chunk: u32 },
    Installed { install_path: String },
    Failed { error: String },
    Updating { progress: f32 },
}

#[derive(Clone, Debug)]
pub struct GameInstallation {
    pub package_id: String,
    pub title_id: String,
    pub title_name: String,
    pub version: String,
    pub install_status: InstallStatus,
    pub install_size: u64,
    pub installed_chunks: Vec<u32>,
    pub install_path: String,
    pub installation_time: u64,
    pub last_played: u64,
    pub play_time_seconds: u64,
    pub achievements_unlocked: Vec<String>,
    pub save_files: Vec<GameSaveData>,
    pub patches: Vec<GamePatch>,
}

#[derive(Clone, Debug)]
pub struct Installer {
    pub installations: HashMap<String, GameInstallation>,
    pub package_cache: HashMap<String, GamePackage>,
    pub memory: VirtualMemory,
    pub install_base_path: String,
    pub max_concurrent_installs: usize,
    pub current_installs: usize,
}

impl Installer {
    pub fn new(memory: VirtualMemory) -> Self {
        Self {
            installations: HashMap::new(),
            package_cache: HashMap::new(),
            memory,
            install_base_path: "/games/".to_string(),
            max_concurrent_installs: 2,
            current_installs: 0,
        }
    }

    pub fn register_package(&mut self, package: GamePackage) -> String {
        let package_id = package.package_id.clone();
        self.package_cache.insert(package_id.clone(), package);
        package_id
    }

    pub fn install_package(
        &mut self,
        package_id: &str,
        profile_id: &str,
    ) -> Result<(), String> {
        let package = self.package_cache
            .get(package_id)
            .ok_or("Package not found")?
            .clone();

        if self.current_installs >= self.max_concurrent_installs {
            return Err("Maximum concurrent installs reached".to_string());
        }

        if self.installations.contains_key(package_id) {
            return Err("Package already installed".to_string());
        }

        self.current_installs += 1;

        let mut installation = GameInstallation {
            package_id: package.package_id.clone(),
            title_id: package.title_id.clone(),
            title_name: package.title_name.clone(),
            version: package.version.clone(),
            install_status: InstallStatus::Installing {
                progress: 0.0,
                current_chunk: 0,
            },
            install_size: package.get_total_size(),
            installed_chunks: Vec::new(),
            install_path: format!(
                "{}/{}/",
                self.install_base_path,
                package.title_id
            ),
            installation_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_played: 0,
            play_time_seconds: 0,
            achievements_unlocked: Vec::new(),
            save_files: Vec::new(),
            patches: Vec::new(),
        };

        // Simula instalação dos chunks
        let total_chunks = package.chunks.len() as u32;
        for (i, chunk) in package.chunks.iter().enumerate() {
            // Verifica integridade
            if !package.verify_integrity() {
                installation.install_status = InstallStatus::Failed {
                    error: format!("Integrity check failed at chunk {}", i)
                };
                self.current_installs -= 1;
                return Err("Integrity check failed".to_string());
            }

            // "Instala" o chunk na memória virtual
            let address = 0x8000_0000 + (i as u64 * 0x1000);
            for (j, &byte) in chunk.data.iter().enumerate() {
                let _ = self.memory.write8(address + j as u64, byte);
            }

            installation.installed_chunks.push(chunk.chunk_id);

            // Atualiza progresso
            let progress = (i + 1) as f32 / total_chunks as f32;
            installation.install_status = InstallStatus::Installing {
                progress: progress * 100.0,
                current_chunk: chunk.chunk_id,
            };
        }

        installation.install_status = InstallStatus::Installed {
            install_path: installation.install_path.clone(),
        };

        let package_id_clone = package.package_id.clone();
        self.installations.insert(package_id_clone, installation);
        self.current_installs -= 1;

        Ok(())
    }

    pub fn apply_patch(
        &mut self,
        package_id: &str,
        patch: GamePatch,
    ) -> Result<(), String> {
        let installation = self.installations
            .get_mut(package_id)
            .ok_or("Installation not found")?;

        installation.install_status = InstallStatus::Updating {
            progress: 0.0,
        };

        let total_changes = patch.changes.len() as f32;
        for (i, change) in patch.changes.iter().enumerate() {
            match change {
                PatchChange::AddFile { path, data } => {
                    // Adiciona arquivo no sistema de arquivos virtual
                    let address = 0xA000_0000 + (i as u64 * 0x1000);
                    for (j, &byte) in data.iter().enumerate() {
                        let _ = self.memory.write8(address + j as u64, byte);
                    }
                    log::info!("Added file: {}", path);
                }
                PatchChange::ModifyFile { path, data } => {
                    log::info!("Modified file: {}", path);
                }
                PatchChange::DeleteFile { path } => {
                    log::info!("Deleted file: {}", path);
                }
                PatchChange::AddChunk { chunk_id, data } => {
                    log::info!("Added chunk: {}", chunk_id);
                }
                PatchChange::ModifyChunk { chunk_id, data } => {
                    log::info!("Modified chunk: {}", chunk_id);
                }
            }

            let progress = (i + 1) as f32 / total_changes * 100.0;
            installation.install_status = InstallStatus::Updating {
                progress,
            };
        }

        installation.version = patch.version.clone();
        installation.patches.push(patch);

        match &installation.install_status {
            InstallStatus::Installed { install_path } => {
                installation.install_status = InstallStatus::Installed {
                    install_path: install_path.clone(),
                };
            }
            _ => {
                installation.install_status = InstallStatus::Failed {
                    error: "Update incomplete".to_string(),
                };
            }
        }

        Ok(())
    }

    pub fn uninstall_package(&mut self, package_id: &str) -> Result<(), String> {
        if !self.installations.contains_key(package_id) {
            return Err("Package not installed".to_string());
        }

        // Libera memória virtual
        let installation = self.installations.get(package_id).unwrap();
        for (i, _) in installation.installed_chunks.iter().enumerate() {
            let address = 0x8000_0000 + (i as u64 * 0x1000);
            for j in 0..0x1000 {
                let _ = self.memory.write8(address + j, 0);
            }
        }

        self.installations.remove(package_id);
        Ok(())
    }

    pub fn get_installation(&self, package_id: &str) -> Option<&GameInstallation> {
        self.installations.get(package_id)
    }

    pub fn get_all_installations(&self) -> Vec<&GameInstallation> {
        self.installations.values().collect()
    }

    pub fn get_installed_games(&self) -> Vec<String> {
        self.installations.values()
            .map(|inst| format!("{} - {}", inst.title_name, inst.version))
            .collect()
    }

    pub fn update_play_time(&mut self, package_id: &str, seconds: u64) -> Result<(), String> {
        let installation = self.installations
            .get_mut(package_id)
            .ok_or("Installation not found")?;
        
        installation.play_time_seconds += seconds;
        installation.last_played = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(())
    }

    pub fn save_game(
        &mut self,
        package_id: &str,
        profile_id: &str,
        save_data: GameSaveData,
    ) -> Result<(), String> {
        let installation = self.installations
            .get_mut(package_id)
            .ok_or("Installation not found")?;

        // Remove save antigo se existir
        installation.save_files.retain(|s| s.profile_id != profile_id);
        
        // Salva dados na memória virtual
        let address = 0xB000_0000 + (installation.save_files.len() as u64 * 0x1000);
        for (i, &byte) in save_data.data.iter().enumerate() {
            let _ = self.memory.write8(address + i as u64, byte);
        }

        installation.save_files.push(save_data);
        Ok(())
    }

    pub fn load_save(
        &self,
        package_id: &str,
        profile_id: &str,
    ) -> Result<Vec<u8>, String> {
        let installation = self.installations
            .get(package_id)
            .ok_or("Installation not found")?;

        let save = installation.save_files
            .iter()
            .find(|s| s.profile_id == profile_id)
            .ok_or("Save not found")?;

        Ok(save.data.clone())
    }
}
