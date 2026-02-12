pub enum KeyboardLayout {
    Qwerty,
    Qwertz,
}

pub struct SystemStatus {
    pub wifi_strength: u8,
    pub weather_icon: u8, // 0-4 as per your design
    pub time_str: String,
}

pub struct Context {
    pub dark_mode: bool,
    pub layout: KeyboardLayout,
    pub status: SystemStatus,
}

impl Context {
    pub fn new() -> Self {
        Self {
            dark_mode: false,
            layout: KeyboardLayout::Qwerty,
            status: SystemStatus {
                wifi_strength: 0,
                weather_icon: 0,
                time_str: String::from("00:00"),
            },
        }
    }
}