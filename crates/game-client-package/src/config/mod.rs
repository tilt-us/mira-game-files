pub mod debug_config;

use std::fs;
use std::path::Path;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// #######################################################
// #                   ClientConfigs                     #
// #######################################################

#[derive(Resource, Deserialize, Serialize, Debug, Clone)]
pub struct ClientConfigs {
    pub config_general: GeneralConfig,
    pub config_graphics: GraphicsConfig,
    pub config_input: InputConfig
}

impl Default for ClientConfigs {
    fn default() -> Self {
        Self {
            config_general: GeneralConfig::default(),
            config_graphics: GraphicsConfig::default(),
            config_input: InputConfig::default()
        }
    }
}

impl ClientConfigs {

    pub fn ensure_config_files_exists() {
        Self::ensure_default_config_file("config/client_general.toml", &GeneralConfig::default());
        Self::ensure_default_config_file("config/client_graphics.toml", &GraphicsConfig::default());
        Self::ensure_default_config_file("config/client_input.toml", &InputConfig::default());
    }

    fn ensure_default_config_file<T: Serialize>(path: &str, default: &T) {
        let config_path = Path::new(path);

        if config_path.exists() {
            return;
        }

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }

        Self::save(default, path);
    }

    pub fn load<T: for<'de> Deserialize<'de>>(path: &str) -> T {
        let content = fs::read_to_string(Path::new(path)).expect("Failed to read config file");
        toml::from_str(&content).expect("Failed to parse toml file")
    }

    pub fn new() -> Self {
        Self::ensure_config_files_exists();

        Self {
            config_general: Self::load("config/client_general.toml"),
            config_graphics: Self::load("config/client_graphics.toml"),
            config_input: Self::load("config/client_input.toml")
        }
    }

    pub fn save_all(&self) {
        Self::ensure_config_files_exists();
        Self::save(&self.config_general, "config/client_general.toml");
        Self::save(&self.config_graphics, "config/client_graphics.toml");
        Self::save(&self.config_input, "config/client_input.toml");
    }

    fn save<T: Serialize>(data: &T, path: &str) {
        let toml_string = toml::to_string_pretty(data).expect("Failed to serialize to TOML");
        fs::write(Path::new(path), toml_string).expect("Failed to write config file");
    }
}

// #######################################################
// #                   GeneralConfig                     #
// #######################################################

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_language")]
    pub language: String
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: default_language()
        }
    }
}

fn default_language() -> String {
    "english".to_string()
}

// #######################################################
// #                  GraphicsConfig                     #
// #######################################################

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GraphicsConfig {
    #[serde(default = "default_window_width")]
    pub window_width: u32,
    #[serde(default = "default_window_height")]
    pub window_height: u32,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default = "default_vsync")]
    pub vsync: String,
    #[serde(default = "default_graphic_backend")]
    pub graphic_backend: String,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: String
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            window_width: default_window_width(),
            window_height: default_window_height(),
            fullscreen: false,
            vsync: default_vsync(),
            graphic_backend: default_graphic_backend(),
            ui_scale: default_ui_scale()
        }
    }

}

fn default_window_width() -> u32 {
    1270
}

fn default_window_height() -> u32 {
    1270
}

fn default_vsync() -> String {
    "default".to_string()
}

fn default_graphic_backend() -> String {
    "AUTO".to_string()
}

fn default_ui_scale() -> String {
    "3".to_string()
}

// #######################################################
// #                    InputConfig                      #
// #######################################################

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InputConfig {
    #[serde(default)]
    movement_forward: String,
    #[serde(default)]
    movement_backward: String,
    #[serde(default)]
    movement_left: String,
    #[serde(default)]
    movement_right: String,
    #[serde(default)]
    movement_jump: String,
    #[serde(default)]
    movement_sprint: String,
    #[serde(default)]
    movement_sneak: String,
    #[serde(default)]
    mouse_sensitivity_vertical: f32,
    #[serde(default)]
    mouse_sensitivity_horizontal: f32,
    #[serde(default)]
    interact: String,
    #[serde(default)]
    open_menu: String,
    #[serde(default)]
    attack: String,
    #[serde(default)]
    close_active_ui: String,
    #[serde(default)]
    system_info_screen: String,
    #[serde(default)]
    debug_collider: String,
    #[serde(default)]
    benchmark_start_stop: String
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            movement_forward: String::from("W"),
            movement_backward: String::from("S"),
            movement_left: String::from("A"),
            movement_right: String::from("D"),
            movement_jump: String::from("Space"),
            movement_sprint: String::from("ShiftLeft"),
            movement_sneak: String::from("CtrlLeft"),
            mouse_sensitivity_vertical: 1.0,
            mouse_sensitivity_horizontal: 1.0,
            interact: String::from("F"),
            open_menu: String::from("Escape"),
            attack: String::from("MouseLeft"),
            close_active_ui: String::from("Escape"),
            system_info_screen: String::from("F3"),
            debug_collider: String::from("F3 + C"),
            benchmark_start_stop: String::from("F3 + B")
        }
    }
}