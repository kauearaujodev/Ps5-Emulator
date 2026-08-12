use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Constantes para tipos de pacotes PS5
pub const PACKAGE_TYPE_GAME: u8 = 0x01;
pub const PACKAGE_TYPE_UPDATE: u8 = 0x02;
pub const PACKAGE_TYPE_DLC: u8 = 0x03;
pub const PACKAGE_TYPE_PATCH: u8 = 0x04;

// Tamanhos padrão de blocos
pub const BLOCK_SIZE: u64 = 4096;
pub const MAX_PACKAGE_SIZE: u64 = 100 * 1024 * 1024 * 1024; // 100 GB

#[derive(Clone, Debug)]
pub struct GamePackage {
    pub package_id: String,
    pub title_id: String,      // CUSA-XXXXX
    pub title_name: String,
    pub version: String,
    pub package_type: u8,
    pub size_bytes: u64,
    pub install_size: u64,
    pub required_firmware: String,
    pub regions: Vec<String>,
    pub languages: Vec<String>,
    pub release_date: u64,     // Timestamp
    pub publisher: String,
    pub developer: String,
    pub genre: String,
    pub rating: String,
    pub description: String,
    pub icon_data: Vec<u8>,
    pub screenshot_previews: Vec<Vec<u8>>,
    pub metadata: HashMap<String, String>,
    pub compressed_data: Vec<u8>,
    pub encryption_key: Option<[u8; 32]>,
    pub signature: Option<Vec<u8>>,
    pub chunks: Vec<PackageChunk>,
    pub is_installed: bool,
    pub install_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PackageChunk {
    pub chunk_id: u32,
    pub offset: u64,
    pub size: u64,
    pub compressed: bool,
    pub checksum: [u8; 32],     // SHA-256
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct GamePatch {
    pub patch_id: String,
    pub target_package_id: String,
    pub version: String,
    pub size_bytes: u64,
    pub changes: Vec<PatchChange>,
    pub is_installed: bool,
}

#[derive(Clone, Debug)]
pub enum PatchChange {
    AddFile { path: String, data: Vec<u8> },
    ModifyFile { path: String, data: Vec<u8> },
    DeleteFile { path: String },
    AddChunk { chunk_id: u32, data: Vec<u8> },
    ModifyChunk { chunk_id: u32, data: Vec<u8> },
}

#[derive(Clone, Debug)]
pub struct GameSaveData {
    pub save_id: String,
    pub package_id: String,
    pub profile_id: String,
    pub save_time: u64,
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub screenshot: Option<Vec<u8>>,
}

impl GamePackage {
    pub fn new(
        title_id: String,
        title_name: String,
        version: String
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            package_id: format!("PKG-{}-{}", title_id, timestamp),
            title_id,
            title_name,
            version,
            package_type: PACKAGE_TYPE_GAME,
            size_bytes: 0,
            install_size: 0,
            required_firmware: "10.00".to_string(),
            regions: vec!["ALL".to_string()],
            languages: vec!["en".to_string(), "pt".to_string()],
            release_date: timestamp,
            publisher: "Unknown".to_string(),
            developer: "Unknown".to_string(),
            genre: "Unknown".to_string(),
            rating: "RP".to_string(),
            description: String::new(),
            icon_data: Vec::new(),
            screenshot_previews: Vec::new(),
            metadata: HashMap::new(),
            compressed_data: Vec::new(),
            encryption_key: None,
            signature: None,
            chunks: Vec::new(),
            is_installed: false,
            install_path: None,
        }
    }

    pub fn add_chunk(&mut self, data: Vec<u8>, compressed: bool) -> u32 {
        let chunk_id = self.chunks.len() as u32;
        let mut chunk = PackageChunk {
            chunk_id,
            offset: self.size_bytes,
            size: data.len() as u64,
            compressed,
            checksum: [0; 32],
            data: data.clone(),
        };

        // Calcula checksum (simplificado)
        let hash = self.calculate_hash(&data);
        chunk.checksum = hash;

        self.size_bytes += data.len() as u64;
        self.chunks.push(chunk);
        chunk_id
    }

    pub fn calculate_hash(&self, data: &[u8]) -> [u8; 32] {
        // SHA-256 simplificado para exemplo
        let mut hash = [0u8; 32];
        for (i, &byte) in data.iter().enumerate() {
            let pos = i % 32;
            hash[pos] = hash[pos].wrapping_add(byte);
            hash[pos] = hash[pos].rotate_left(1);
        }
        hash
    }

    pub fn verify_integrity(&self) -> bool {
        for chunk in &self.chunks {
            let calculated = self.calculate_hash(&chunk.data);
            if calculated != chunk.checksum {
                return false;
            }
        }
        true
    }

    pub fn get_total_size(&self) -> u64 {
        self.chunks.iter().map(|c| c.size).sum()
    }

    pub fn extract_data(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for chunk in &self.chunks {
            result.extend_from_slice(&chunk.data);
        }
        result
    }
}

impl GamePatch {
    pub fn new(target_package_id: String, version: String) -> Self {
        Self {
            patch_id: format!("PATCH-{}-{}", target_package_id, version),
            target_package_id,
            version,
            size_bytes: 0,
            changes: Vec::new(),
            is_installed: false,
        }
    }

    pub fn add_change(&mut self, change: PatchChange) {
        match &change {
            PatchChange::AddFile { data, .. } => {
                self.size_bytes += data.len() as u64;
            }
            PatchChange::ModifyFile { data, .. } => {
                self.size_bytes += data.len() as u64;
            }
            PatchChange::DeleteFile { .. } => {
                // Não adiciona tamanho
            }
            PatchChange::AddChunk { data, .. } => {
                self.size_bytes += data.len() as u64;
            }
            PatchChange::ModifyChunk { data, .. } => {
                self.size_bytes += data.len() as u64;
            }
        }
        self.changes.push(change);
    }
}

impl GameSaveData {
    pub fn new(package_id: String, profile_id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            save_id: format!("SAVE-{}-{}", package_id, timestamp),
            package_id,
            profile_id,
            save_time: timestamp,
            data: Vec::new(),
            metadata: HashMap::new(),
            screenshot: None,
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}
