use egui::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceType {
    Phone,
    Tablet,
    Desktop,
    Chromebook,
}

#[derive(Clone, Debug)]
pub struct ScreenInfo {
    pub width: f32,
    pub height: f32,
    pub device: DeviceType,
    pub is_touch: bool,
    pub scale_factor: f32,
    pub orientation: ScreenOrientation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScreenOrientation {
    Portrait,
    Landscape,
}

impl ScreenInfo {
    pub fn new(ctx: &egui::Context) -> Self {
        let screen_rect = ctx.screen_rect();
        let width = screen_rect.width();
        let height = screen_rect.height();
        
        let is_touch = ctx.input(|i| i.touch.is_some());
        let scale_factor = ctx.pixels_per_point();
        
        let orientation = if height > width {
            ScreenOrientation::Portrait
        } else {
            ScreenOrientation::Landscape
        };
        
        let device = Self::detect_device(width, height, is_touch);
        
        Self {
            width,
            height,
            device,
            is_touch,
            scale_factor,
            orientation,
        }
    }

    fn detect_device(width: f32, height: f32, is_touch: bool) -> DeviceType {
        let min_dim = width.min(height);
        
        if is_touch && min_dim < 600.0 {
            DeviceType::Phone
        } else if is_touch && min_dim < 1024.0 {
            DeviceType::Tablet
        } else if width >= 1366.0 && height >= 768.0 {
            if width >= 1920.0 {
                DeviceType::Desktop
            } else {
                DeviceType::Chromebook
            }
        } else {
            DeviceType::Desktop
        }
    }

    pub fn is_phone(&self) -> bool {
        self.device == DeviceType::Phone
    }

    pub fn is_tablet(&self) -> bool {
        self.device == DeviceType::Tablet
    }

    pub fn is_desktop(&self) -> bool {
        self.device == DeviceType::Desktop
    }

    pub fn is_chromebook(&self) -> bool {
        self.device == DeviceType::Chromebook
    }

    pub fn is_portrait(&self) -> bool {
        self.orientation == ScreenOrientation::Portrait
    }

    pub fn get_grid_cols(&self) -> usize {
        match (&self.device, &self.orientation) {
            (DeviceType::Phone, ScreenOrientation::Portrait) => 2,
            (DeviceType::Phone, ScreenOrientation::Landscape) => 3,
            (DeviceType::Tablet, _) => 4,
            (DeviceType::Chromebook, _) => 4,
            (DeviceType::Desktop, _) => 6,
        }
    }

    pub fn get_button_size(&self) -> f32 {
        match self.device {
            DeviceType::Phone => if self.is_portrait() { 70.0 } else { 55.0 },
            DeviceType::Tablet => 60.0,
            DeviceType::Chromebook => 50.0,
            DeviceType::Desktop => 45.0,
        }
    }

    pub fn get_font_size(&self) -> f32 {
        match self.device {
            DeviceType::Phone => if self.is_portrait() { 16.0 } else { 14.0 },
            DeviceType::Tablet => 18.0,
            DeviceType::Chromebook => 16.0,
            DeviceType::Desktop => 20.0,
        }
    }

    pub fn get_padding(&self) -> f32 {
        match self.device {
            DeviceType::Phone => 8.0,
            DeviceType::Tablet => 12.0,
            DeviceType::Chromebook => 10.0,
            DeviceType::Desktop => 16.0,
        }
    }

    pub fn get_game_card_size(&self) -> Vec2 {
        match self.device {
            DeviceType::Phone => {
                if self.is_portrait() {
                    Vec2::new(160.0, 220.0)
                } else {
                    Vec2::new(140.0, 190.0)
                }
            }
            DeviceType::Tablet => Vec2::new(180.0, 240.0),
            DeviceType::Chromebook => Vec2::new(200.0, 260.0),
            DeviceType::Desktop => Vec2::new(220.0, 280.0),
        }
    }

    pub fn get_control_spacing(&self) -> f32 {
        match self.device {
            DeviceType::Phone => 15.0,
            DeviceType::Tablet => 25.0,
            DeviceType::Chromebook => 20.0,
            DeviceType::Desktop => 30.0,
        }
    }

    pub fn get_control_size(&self) -> f32 {
        match self.device {
            DeviceType::Phone => {
                if self.is_portrait() {
                    self.width / 5.0
                } else {
                    self.height / 4.0
                }
            }
            DeviceType::Tablet => {
                if self.is_portrait() {
                    self.width / 6.0
                } else {
                    self.height / 4.5
                }
            }
            DeviceType::Chromebook => self.width / 8.0,
            DeviceType::Desktop => self.width / 10.0,
        }
    }
          }
