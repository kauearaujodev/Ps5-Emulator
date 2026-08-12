use std::collections::HashMap;
use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct SoundSource3D {
    pub position: Vector3D,
    pub velocity: Vector3D,
    pub volume: f32,
    pub attenuation: f32,
    pub sound_type: SoundType,
}

#[derive(Clone, Debug)]
pub struct Vector3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SoundType {
    Positional,
    Ambient,
    Music,
    Voice,
    UI,
}

#[derive(Clone, Debug)]
pub struct Listener3D {
    pub position: Vector3D,
    pub orientation: Vector3D,
    pub up: Vector3D,
}

pub struct Tempest3DEngine {
    pub listener: Listener3D,
    pub sources: HashMap<String, SoundSource3D>,
    pub output_channels: u8,
}

impl Tempest3DEngine {
    pub fn new() -> Self {
        Self {
            listener: Listener3D {
                position: Vector3D { x: 0.0, y: 0.0, z: 0.0 },
                orientation: Vector3D { x: 0.0, y: 0.0, z: -1.0 },
                up: Vector3D { x: 0.0, y: 1.0, z: 0.0 },
            },
            sources: HashMap::new(),
            output_channels: 2,
        }
    }

    pub fn add_source(&mut self, id: &str, source: SoundSource3D) {
        self.sources.insert(id.to_string(), source);
    }

    pub fn update_source(&mut self, id: &str, position: Vector3D, velocity: Vector3D) {
        if let Some(source) = self.sources.get_mut(id) {
            source.position = position;
            source.velocity = velocity;
        }
    }

    pub fn get_volume_at_listener(&self, source_id: &str) -> f32 {
        if let Some(source) = self.sources.get(source_id) {
            let distance = self.calculate_distance(&source.position);
            let volume = 1.0 / (1.0 + distance * source.attenuation);
            volume * source.volume
        } else {
            0.0
        }
    }

    pub fn get_stereo_pan(&self, source_id: &str) -> (f32, f32) {
        if let Some(source) = self.sources.get(source_id) {
            let dx = source.position.x - self.listener.position.x;
            let dz = source.position.z - self.listener.position.z;
            
            let angle = dz.atan2(dx);
            let pan = (angle / (PI / 2.0)).clamp(-1.0, 1.0);
            
            let left = (1.0 - pan) / 2.0;
            let right = (1.0 + pan) / 2.0;
            
            (left, right)
        } else {
            (0.5, 0.5)
        }
    }

    pub fn calculate_distance(&self, position: &Vector3D) -> f32 {
        let dx = position.x - self.listener.position.x;
        let dy = position.y - self.listener.position.y;
        let dz = position.z - self.listener.position.z;
        
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn calculate_doppler(&self, source_id: &str) -> f32 {
        if let Some(source) = self.sources.get(source_id) {
            let distance = self.calculate_distance(&source.position);
            let speed_of_sound = 343.0;
            
            let vr = (source.velocity.x * (source.position.x - self.listener.position.x) +
                     source.velocity.y * (source.position.y - self.listener.position.y) +
                     source.velocity.z * (source.position.z - self.listener.position.z)) / distance;
            
            1.0 + vr / speed_of_sound
        } else {
            1.0
        }
    }
  }
