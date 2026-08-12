use egui::*;
use super::engine::AudioEngine;
use std::sync::Arc;

pub struct AudioController {
    pub engine: Arc<AudioEngine>,
    pub show_controls: bool,
    pub music_playing: bool,
}

impl AudioController {
    pub fn new(engine: Arc<AudioEngine>) -> Self {
        Self {
            engine,
            show_controls: true,
            music_playing: false,
        }
    }

    pub fn render(&mut self, ui: &mut Ui) {
        if !self.show_controls {
            return;
        }

        ui.group(|ui| {
            ui.label(RichText::new("🎵 Áudio").size(18.0).strong());
            ui.add_space(10.0);

            // Master Volume
            ui.horizontal(|ui| {
                ui.label("🔊 Master:");
                let vol = self.engine.volume.lock().unwrap().master;
                let mut new_vol = vol;
                ui.add(egui::Slider::new(&mut new_vol, 0.0..=1.0).text(""));
                if new_vol != vol {
                    self.engine.set_master_volume(new_vol);
                }
                ui.label(format!("{:.0}%", new_vol * 100.0));
            });

            // Music Volume
            ui.horizontal(|ui| {
                ui.label("🎵 Música:");
                let vol = self.engine.volume.lock().unwrap().music;
                let mut new_vol = vol;
                ui.add(egui::Slider::new(&mut new_vol, 0.0..=1.0).text(""));
                if new_vol != vol {
                    self.engine.set_music_volume(new_vol);
                }
                ui.label(format!("{:.0}%", new_vol * 100.0));
            });

            // SFX Volume
            ui.horizontal(|ui| {
                ui.label("💥 Efeitos:");
                let vol = self.engine.volume.lock().unwrap().sfx;
                let mut new_vol = vol;
                ui.add(egui::Slider::new(&mut new_vol, 0.0..=1.0).text(""));
                if new_vol != vol {
                    self.engine.set_sfx_volume(new_vol);
                }
                ui.label(format!("{:.0}%", new_vol * 100.0));
            });

            ui.add_space(10.0);

            // Botões de teste
            ui.horizontal(|ui| {
                if ui.button("🔊 Teste Som").clicked() {
                    self.play_test_sound();
                }

                if ui.button(self.music_playing ? "⏹ Parar" : "▶ Tocar Música").clicked() {
                    if self.music_playing {
                        self.engine.stop_music();
                        self.music_playing = false;
                    } else {
                        self.play_test_music();
                    }
                }

                if ui.button("⏹ Parar Tudo").clicked() {
                    self.engine.stop_all();
                    self.music_playing = false;
                }
            });
        });
    }

    fn play_test_sound(&self) {
        let samples = crate::audio::loader::AudioLoader::generate_test_tone(440, 0.5, 48000);
        let _ = self.engine.play_sound("test", samples);
    }

    fn play_test_music(&self) {
        let samples = crate::audio::loader::AudioLoader::generate_test_tone(261, 2.0, 48000);
        let _ = self.engine.play_music(samples, true);
        self.music_playing = true;
    }
}
