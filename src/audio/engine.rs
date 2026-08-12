use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rodio::{OutputStream, Sink, Source};
use super::loader::*;

#[derive(Clone)]
pub struct AudioEngine {
    pub initialized: Arc<Mutex<bool>>,
    pub stream: Arc<Mutex<Option<OutputStream>>>,
    pub sinks: Arc<Mutex<HashMap<String, Sink>>>,
    pub volume: Arc<Mutex<AudioVolume>>,
    pub loader: Arc<Mutex<AudioLoader>>,
}

#[derive(Clone, Debug)]
pub struct AudioVolume {
    pub master: f32,
    pub music: f32,
    pub sfx: f32,
    pub voice: f32,
    pub ambient: f32,
}

impl Default for AudioVolume {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.8,
            sfx: 0.9,
            voice: 1.0,
            ambient: 0.6,
        }
    }
}

impl AudioEngine {
    pub fn new() -> Self {
        let loader = AudioLoader::new();
        
        Self {
            initialized: Arc::new(Mutex::new(false)),
            stream: Arc::new(Mutex::new(None)),
            sinks: Arc::new(Mutex::new(HashMap::new())),
            volume: Arc::new(Mutex::new(AudioVolume::default())),
            loader: Arc::new(Mutex::new(loader)),
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        if *self.initialized.lock().unwrap() {
            return Ok(());
        }

        match OutputStream::try_default() {
            Ok((stream, _handle)) => {
                *self.stream.lock().unwrap() = Some(stream);
                *self.initialized.lock().unwrap() = true;
                println!("🎵 Áudio inicializado com sucesso!");
                Ok(())
            }
            Err(e) => Err(format!("Erro ao inicializar áudio: {}", e)),
        }
    }

    pub fn play_sound(&self, name: &str, data: Vec<u8>) -> Result<(), String> {
        if !*self.initialized.lock().unwrap() {
            return Err("Áudio não inicializado".to_string());
        }

        let stream_handle = self.get_handle()?;
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("Erro ao criar sink: {}", e))?;

        // Converte dados para som
        let source = AudioLoader::load_source(data)?;
        sink.append(source);

        // Aplica volume
        let volume = self.volume.lock().unwrap();
        sink.set_volume(volume.master * volume.sfx);

        // Armazena sink com nome único
        let sink_id = format!("{}_{}", name, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros());
        
        self.sinks.lock().unwrap().insert(sink_id, sink);

        Ok(())
    }

    pub fn play_music(&self, data: Vec<u8>, looped: bool) -> Result<(), String> {
        if !*self.initialized.lock().unwrap() {
            return Err("Áudio não inicializado".to_string());
        }

        // Para música atual
        self.stop_music();

        let stream_handle = self.get_handle()?;
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("Erro ao criar sink: {}", e))?;

        let source = AudioLoader::load_source(data)?;
        
        if looped {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        let volume = self.volume.lock().unwrap();
        sink.set_volume(volume.master * volume.music);

        self.sinks.lock().unwrap().insert("music".to_string(), sink);

        Ok(())
    }

    pub fn stop_music(&self) {
        if let Some(sink) = self.sinks.lock().unwrap().remove("music") {
            sink.stop();
        }
    }

    pub fn stop_sound(&self, name: &str) {
        let mut sinks = self.sinks.lock().unwrap();
        let to_remove: Vec<String> = sinks.keys()
            .filter(|k| k.starts_with(name))
            .cloned()
            .collect();
        
        for key in to_remove {
            if let Some(sink) = sinks.remove(&key) {
                sink.stop();
            }
        }
    }

    pub fn stop_all(&self) {
        let mut sinks = self.sinks.lock().unwrap();
        for (_, sink) in sinks.drain() {
            sink.stop();
        }
    }

    pub fn set_master_volume(&self, volume: f32) {
        let mut vol = self.volume.lock().unwrap();
        vol.master = volume.clamp(0.0, 1.0);
        self.update_all_volumes();
    }

    pub fn set_music_volume(&self, volume: f32) {
        let mut vol = self.volume.lock().unwrap();
        vol.music = volume.clamp(0.0, 1.0);
        self.update_music_volume();
    }

    pub fn set_sfx_volume(&self, volume: f32) {
        let mut vol = self.volume.lock().unwrap();
        vol.sfx = volume.clamp(0.0, 1.0);
        self.update_sfx_volumes();
    }

    fn update_all_volumes(&self) {
        self.update_music_volume();
        self.update_sfx_volumes();
    }

    fn update_music_volume(&self) {
        let volume = self.volume.lock().unwrap();
        let total = volume.master * volume.music;
        
        if let Some(sink) = self.sinks.lock().unwrap().get("music") {
            sink.set_volume(total);
        }
    }

    fn update_sfx_volumes(&self) {
        let volume = self.volume.lock().unwrap();
        let total = volume.master * volume.sfx;
        
        let sinks = self.sinks.lock().unwrap();
        for (name, sink) in sinks.iter() {
            if name != "music" {
                sink.set_volume(total);
            }
        }
    }

    fn get_handle(&self) -> Result<rodio::OutputStreamHandle, String> {
        if let Some(stream) = self.stream.lock().unwrap().as_ref() {
            Ok(stream.1.clone())
        } else {
            Err("Dispositivo de áudio não disponível".to_string())
        }
    }
}
