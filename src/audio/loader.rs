use rodio::{Decoder, Source};
use std::io::Cursor;

pub struct AudioLoader;

impl AudioLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load_source(data: Vec<u8>) -> Result<impl Source<Item = i16> + Send + Sync, String> {
        // Tenta carregar como WAV
        if let Ok(decoder) = Decoder::new(Cursor::new(data.clone())) {
            return Ok(decoder);
        }

        // Tenta carregar como MP3/FLAC
        if let Ok(decoder) = Decoder::new(Cursor::new(data)) {
            return Ok(decoder);
        }

        Err("Formato de áudio não suportado".to_string())
    }

    pub fn generate_test_tone(frequency: u32, duration: f32, sample_rate: u32) -> Vec<u8> {
        let num_samples = (sample_rate as f32 * duration) as usize;
        let mut samples = Vec::with_capacity(num_samples * 2);
        
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let sample = (t * frequency as f32 * 2.0 * std::f32::consts::PI).sin();
            let sample_i16 = (sample * 32767.0) as i16;
            samples.extend_from_slice(&sample_i16.to_le_bytes());
        }
        
        samples
    }
}
