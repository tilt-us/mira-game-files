pub mod debug_config;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// #######################################################
// #                   ClientConfigs                     #
// #######################################################

/// Represents the configuration settings for a client application.
///
/// This struct serves as a container for various configuration categories,
/// including general settings, graphics settings, and input settings.
/// It is marked with several derive macros to enable functionality such as
/// resource management, serialization, deserialization, debugging, and cloning.
///
/// # Fields
///
/// - `config_general`: Contains general configuration settings that apply
///   to the overall client behavior. The specific structure and data are
///   defined in `GeneralConfig`.
///
/// - `config_graphics`: Contains configuration settings related to graphics
///   and rendering. The specifics are captured in the `GraphicsConfig` struct.
///
/// - `config_input`: Contains configuration settings for input handling,
///   such as keybindings or controller mappings. The details are in
///   the `InputConfig` struct.
///
/// # Derive Macros
///
/// - `Resource`: Indicates that this struct can be registered as a resource
///   in an ECS (Entity Component System) framework.
/// - `Deserialize` and `Serialize`: Allow this struct to be serialized to
///   and deserialized from formats such as JSON or TOML.
/// - `Debug`: Enables debug
#[derive(Resource, Deserialize, Serialize, Debug, Clone)]
pub struct ClientConfigs {
    pub config_general: GeneralConfig,
    pub config_graphics: GraphicsConfig,
    pub config_input: InputConfig,
}

impl Default for ClientConfigs {
    /// Creates a new instance of the struct with default values for all configuration fields.
    ///
    /// # Returns
    /// A new instance of the struct with the following default configurations
    fn default() -> Self {
        Self {
            config_general: GeneralConfig::default(),
            config_graphics: GraphicsConfig::default(),
            config_input: InputConfig::default(),
        }
    }
}

impl ClientConfigs {
    /// Ensures the existence of default configuration files for the application.
    ///
    /// This function checks whether the specified configuration files exist in their respective
    /// locations. If any of the files are missing, it creates them with default values provided
    /// by their respective configuration structs.
    ///
    /// The following configuration files are validated and created if necessary:
    /// - `config/client_general.toml`: Initialized with default values from `GeneralConfig::default()`.
    /// - `config/client_graphics.toml`: Initialized with default values from `GraphicsConfig::default()`.
    /// - `config/client_input.toml`: Initialized with default values from `InputConfig::default()`.
    ///
    /// # Panics
    /// This function may panic if there is an error during the file creation or if the default
    /// configuration values for any of the files cannot be serialized.
    pub fn ensure_config_files_exists() {
        Self::ensure_default_config_file("config/client_general.toml", &GeneralConfig::default());
        Self::ensure_default_config_file("config/client_graphics.toml", &GraphicsConfig::default());
        Self::ensure_default_config_file("config/client_input.toml", &InputConfig::default());
    }

    /// Ensures the existence of a default configuration file at the specified path.
    ///
    /// # Description
    /// This function checks whether a configuration file exists at the provided path.
    /// If the file does not exist, it creates any necessary parent directories and writes
    /// a default configuration to the specified path. The default configuration is provided
    /// as a serializable object.
    ///
    /// # Type Parameters
    /// - `T`: A type that implements the `Serialize` trait, representing the default configuration data.
    ///
    /// # Arguments
    /// - `path`: A string slice that specifies the file path where the configuration file should exist.
    /// - `default`: A reference to an object of type `T` containing the default configuration data to be saved.
    ///
    /// # Panics
    /// - The function will panic if it fails to create the required parent directories.
    /// - Any errors encountered during the serialization or saving process will be handled
    ///   by the `Self::save` method, which is assumed to manage those errors.
    ///
    /// # Note
    /// - The `Self::save` method is not defined within this snippet. Ensure that it is implemented
    ///   appropriately to handle serialization and writing logic.
    /// - This function assumes the use of `serde` for serialization and requires `T` to derive or implement
    ///   the `Serialize` trait.
    ///
    /// # Dependencies
    /// - `std::path::Path`
    /// - `std::fs`
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

    /// Loads and deserializes a TOML configuration file into a specified type.
    ///
    /// # Type Parameters
    /// * `T` - The type into which the TOML file should be deserialized.
    ///          Must implement the `Deserialize` trait for deserialization to succeed.
    ///
    /// # Parameters
    /// * `path` - A string slice representing the file path to the TOML configuration file.
    ///
    /// # Returns
    /// * This function returns an instance of type `T` that is populated with the data
    ///   deserialized from the TOML file.
    ///
    /// # Panics
    /// * The function will panic if:
    ///     1. The file doesn't exist or cannot be read.
    ///     2. The file contents cannot be parsed as a valid TOML structure into the specified type `T`.
    pub fn load<T: for<'de> Deserialize<'de>>(path: &str) -> T {
        let content = fs::read_to_string(Path::new(path)).expect("Failed to read config file");
        toml::from_str(&content).expect("Failed to parse toml file")
    }

    /// Creates a new instance of the struct and initializes its configuration by ensuring
    /// the existence of required configuration files and loading their content.
    ///
    /// # Behavior
    ///  - `ensure_config_files_exists` to verify that necessary configuration files exist.
    ///     If they don't exist, this function typically creates them or raises an appropriate error.
    ///     Loads the configuration data from three specific TOML files located in the `config` directory:
    ///  - `config/client_general.toml` for general client settings.
    ///  - `config/client_graphics.toml` for client graphics settings.
    ///  - `config/client_input.toml` for client input settings.
    ///
    /// # Returns
    ///    A new instance of the struct, populated with the loaded configuration data.
    ///
    /// # Panics
    ///  - This function may panic if it fails to load the configuration files or if they are
    ///    missing required fields.
    pub fn new() -> Self {
        Self::ensure_config_files_exists();

        Self {
            config_general: Self::load("config/client_general.toml"),
            config_graphics: Self::load("config/client_graphics.toml"),
            config_input: Self::load("config/client_input.toml"),
        }
    }

    /// Saves all configuration data to their respective files.
    ///
    /// This method ensures that the necessary configuration files exist
    /// before proceeding to save the data. Each configuration file is
    /// saved in its corresponding designated path:
    ///
    /// - `config/client_general.toml` for general configuration.
    /// - `config/client_graphics.toml` for graphics configuration.
    /// - `config/client_input.toml` for input configuration.
    ///
    /// # Behavior
    /// - Ensures the existence of the configuration files by calling `ensure_config_files_exists`.
    /// - Saves the corresponding configuration data into the respective file paths.
    ///
    /// This function is intended to persist the application's configuration settings
    /// so that they can be reloaded in subsequent runs of the application.
    pub fn save_all(&self) {
        Self::ensure_config_files_exists();
        Self::save(&self.config_general, "config/client_general.toml");
        Self::save(&self.config_graphics, "config/client_graphics.toml");
        Self::save(&self.config_input, "config/client_input.toml");
    }

    /// Saves a serializable data structure to a file in TOML format.
    ///
    /// # Type Parameters
    /// * `T`: A type that implements the `Serialize` trait, enabling it to be serialized into TOML format.
    ///
    /// # Parameters
    /// * `data`: A reference to the data structure that will be serialized and saved.
    /// * `path`: The file path where the serialized TOML data will be written.
    ///
    /// # Panics
    /// * This function panics if the serialization to the TOML format fails.
    /// * This function also panics if writing the serialized data to the specified file path fails.
    fn save<T: Serialize>(data: &T, path: &str) {
        let toml_string = toml::to_string_pretty(data).expect("Failed to serialize to TOML");
        fs::write(Path::new(path), toml_string).expect("Failed to write config file");
    }
}

// #######################################################
// #                   GeneralConfig                     #
// #######################################################

/// Represents the general configuration settings for an application.
///
/// This structure is used to store general configuration details such as the
/// default language. It derives several traits to enable functionalities like
/// serialization, deserialization, debugging, and cloning.
///
/// # Fields
///
/// * `language` - A `String` that specifies the language setting for the
///   application.
///   - If not explicitly provided during deserialization, it will default to
///     the value returned by the `default_language` function.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for GeneralConfig {
    /// Creates a default instance of the struct with pre-defined values.
    ///
    /// # Returns
    ///
    /// An instance of `Self` with its `language` field initialized to the value
    /// returned by the `default_language` function.
    fn default() -> Self {
        Self {
            language: default_language(),
        }
    }
}

/// Returns the default language as a `String`.
///
/// # Details
/// The default language returned by this function is `"english"`.
fn default_language() -> String {
    "english".to_string()
}

// #######################################################
// #                  GraphicsConfig                     #
// #######################################################

/// Represents the configuration settings for graphics in an application.
///
/// This struct is used to configure various graphical parameters such as
/// window dimensions, fullscreen mode, vsync settings, and graphic backend.
///
/// # Fields
///
/// * `window_width` (*u32*): The width of the application window in pixels. This field uses
///   a default value provided by the `default_window_width` function if not specified.
///
/// * `window_height` (*u32*): The height of the application window in pixels. This field uses
///   a default value provided by the `default_window_height` function if not specified.
///
/// * `fullscreen` (*bool*): Whether the application should run in fullscreen mode. Defaults to `false`
///   if not specified.
///
/// * `vsync` (*String*): The vertical synchronization setting. This field defaults to the value
///   provided by the `default_vsync` function if not specified.
///
/// * `graphic_backend` (*String*): Specifies the graphical backend to use (e.g., "Vulkan", "OpenGL", etc.).
///   Defaults to the value provided by the `default_graphic_backend` function if not specified.
///
/// * `ui_scale` (*String*): The scaling factor for the user interface. This value is used to scale
///   UI elements. Defaults to the value provided by the `default_ui_scale` function if not specified.
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
    pub ui_scale: String,
}

impl Default for GraphicsConfig {
    /// Provides a default implementation for initializing an instance of the struct.
    ///
    /// This function returns a new instance of the struct with default values for its fields:
    /// - `window_width`: Set using the `default_window_width()` function, which provides
    ///   the default width of the application window.
    /// - `window_height`: Set using the `default_window_height()` function, which provides
    ///   the default height of the application window.
    /// - `fullscreen`: Set to `false`, indicating that the application window is not in fullscreen mode by default.
    /// - `vsync`: Set using the `default_vsync()` function, determining whether vertical synchronization is enabled.
    /// - `graphic_backend`: Set using the `default_graphic_backend()` function, specifying the default graphics backend.
    /// - `ui_scale`: Set using the `default_ui_scale()` function to define the default scaling factor for the user interface.
    ///
    /// # Returns
    /// A new instance of `Self` with default configuration values.
    fn default() -> Self {
        Self {
            window_width: default_window_width(),
            window_height: default_window_height(),
            fullscreen: false,
            vsync: default_vsync(),
            graphic_backend: default_graphic_backend(),
            ui_scale: default_ui_scale(),
        }
    }
}

/// Returns the default window width as a `u32`.
///
/// # Details
/// The default window width returned by this function is `1270`.
fn default_window_width() -> u32 {
    1270
}

/// Returns the default window height as a `u32`.
///
/// # Details
/// The default window height returned by this function is `720`.
fn default_window_height() -> u32 {
    720
}

/// Returns the default vsync mode as a `String`.
///
/// # Details
/// The default vsync mode returned by this function is `"default"` which means Fifo for bevy.
fn default_vsync() -> String {
    "default".to_string()
}

/// Returns the default backend mode as a `String`.
///
/// # Details
/// The default backend mode returned by this function is `"AUTO"` which means PRIMARY for bevy.
fn default_graphic_backend() -> String {
    "AUTO".to_string()
}

/// Returns the default ui scale as a `String`.
///
/// # Details
/// The default ui scale returned by this function is `"3"`.
fn default_ui_scale() -> String {
    "3".to_string()
}

// #######################################################
// #                    InputConfig                      #
// #######################################################

/// A configuration structure for input bindings and settings in a game or application.
///
/// This struct defines key bindings and sensitivity settings for various actions typically
/// used in games, such as movement, interaction, and menu navigation. It supports serialization
/// and deserialization using Serde, making it easy to save or load configurations.
///
/// # Fields
///
/// - `movement_forward` (`String`, default: empty):
///   Key binding for the "Move Forward" action.
///
/// - `movement_backward` (`String`, default: empty):
///   Key binding for the "Move Backward" action.
///
/// - `movement_left` (`String`, default: empty):
///   Key binding for the "Move Left" action.
///
/// - `movement_right` (`String`, default: empty):
///   Key binding for the "Move Right" action.
///
/// - `movement_jump` (`String`, default: empty):
///   Key binding for the "Jump" action.
///
/// - `movement_sprint` (`String`, default: empty):
///   Key binding for the "Sprint" action.
///
/// - `movement_sneak` (`String`, default: empty):
///   Key binding for the "Sneak" or "Crouch" action.
///
/// - `mouse_sensitivity_vertical` (`f32`, default: 0.0):
///   Sensitivity setting for vertical mouse movement.
///
/// - `mouse_sensitivity_horizontal` (`f32`, default: 0.0):
///   Sensitivity setting for horizontal mouse movement.
///
/// - `interact` (`String`, default: empty):
///   Key binding for the "Interact" action, typically used for interacting with objects or NPCs.
///
/// - `open_menu` (`String`, default: empty):
///   Key binding for opening the main game or pause menu.
///
/// - `attack` (`String`, default: empty):
///   Key binding for the "Attack" action, often used for combat.
///
/// - `close_active_ui` (`String`, default: empty):
///   Key binding for closing the active UI screen, such as inventory or menus.
///
/// - `system_info_screen` (`String`, default: empty):
///   Key binding for toggling the system information screen (e.g., debugging stats).
///
/// - `debug_collider` (`String`, default: empty):
///   Key binding for toggling collision debugging visualization.
///
/// - `benchmark_start_stop` (`String`, default: empty):
///   Key binding for starting or stopping a benchmarking session.
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
    party_slot_01: String,
    #[serde(default)]
    party_slot_02: String,
    #[serde(default)]
    party_slot_03: String,
    #[serde(default)]
    party_slot_04: String,
    #[serde(default)]
    party_next_slot: String,
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
    benchmark_start_stop: String,
}

impl InputConfig {
    /// Returns the binding used for forward movement.
    pub fn movement_forward(&self) -> &str {
        &self.movement_forward
    }

    /// Returns the binding used for backward movement.
    pub fn movement_backward(&self) -> &str {
        &self.movement_backward
    }

    /// Returns the binding used for left movement.
    pub fn movement_left(&self) -> &str {
        &self.movement_left
    }

    /// Returns the binding used for right movement.
    pub fn movement_right(&self) -> &str {
        &self.movement_right
    }

    /// Returns the binding used for jumping.
    pub fn movement_jump(&self) -> &str {
        &self.movement_jump
    }

    /// Returns the binding used for sprinting.
    pub fn movement_sprint(&self) -> &str {
        &self.movement_sprint
    }

    /// Returns the binding used for sneaking.
    pub fn movement_sneak(&self) -> &str {
        &self.movement_sneak
    }

    /// Returns the binding used for party slot 01.
    pub fn party_slot_01(&self) -> &str {
        &self.party_slot_01
    }

    /// Returns the binding used for party slot 02.
    pub fn party_slot_02(&self) -> &str {
        &self.party_slot_02
    }

    /// Returns the binding used for party slot 03.
    pub fn party_slot_03(&self) -> &str {
        &self.party_slot_03
    }

    /// Returns the binding used for party slot 04.
    pub fn party_slot_04(&self) -> &str {
        &self.party_slot_04
    }

    /// Returns the binding used for selecting the next available party slot.
    pub fn party_next_slot(&self) -> &str {
        &self.party_next_slot
    }

    /// Returns the binding used for opening the menu.
    pub fn open_menu(&self) -> &str {
        &self.open_menu
    }

    /// Returns the binding used for toggling the system info screen.
    pub fn system_info_screen(&self) -> &str {
        &self.system_info_screen
    }

    /// Returns horizontal mouse sensitivity.
    pub fn mouse_sensitivity_horizontal(&self) -> f32 {
        self.mouse_sensitivity_horizontal
    }

    /// Returns vertical mouse sensitivity.
    pub fn mouse_sensitivity_vertical(&self) -> f32 {
        self.mouse_sensitivity_vertical
    }
}

impl Default for InputConfig {
    /// Provides the default key bindings and control settings for a game or application.
    ///
    /// # Returns
    /// A new instance of the struct with the following predefined default values:
    /// - `movement_forward`: "W" - Key for moving forward.
    /// - `movement_backward`: "S" - Key for moving backward.
    /// - `movement_left`: "A" - Key for moving left.
    /// - `movement_right`: "D" - Key for moving right.
    /// - `movement_jump`: "Space" - Key for jumping.
    /// - `movement_sprint`: "ShiftLeft" - Key for sprinting.
    /// - `movement_sneak`: "CtrlLeft" - Key for sneaking or crouching.
    /// - `mouse_sensitivity_vertical`: `1.0` - Default mouse sensitivity for vertical movement.
    /// - `mouse_sensitivity_horizontal`: `1.0` - Default mouse sensitivity for horizontal movement.
    /// - `interact`: "F" - Key for interacting with objects.
    /// - `open_menu`: "Escape" - Key for opening the in-game menu.
    /// - `attack`: "MouseLeft" - Key for attacking or performing primary actions.
    /// - `close_active_ui`: "Escape" - Key for closing active UI elements.
    /// - `system_info_screen`: "F3" - Key to display system information.
    /// - `debug_collider`: "F3 + C" - Key combination to display debug collider information.
    /// - `benchmark_start_stop`: "F3 + B" - Key combination to start/stop benchmarking.
    fn default() -> Self {
        Self {
            movement_forward: String::from("W"),
            movement_backward: String::from("S"),
            movement_left: String::from("A"),
            movement_right: String::from("D"),
            movement_jump: String::from("Space"),
            movement_sprint: String::from("ShiftLeft"),
            movement_sneak: String::from("CtrlLeft"),
            party_slot_01: String::from("1"),
            party_slot_02: String::from("2"),
            party_slot_03: String::from("3"),
            party_slot_04: String::from("4"),
            party_next_slot: String::from("Q"),
            mouse_sensitivity_vertical: 1.0,
            mouse_sensitivity_horizontal: 1.0,
            interact: String::from("F"),
            open_menu: String::from("Escape"),
            attack: String::from("MouseLeft"),
            close_active_ui: String::from("Escape"),
            system_info_screen: String::from("F3"),
            debug_collider: String::from("F3 + C"),
            benchmark_start_stop: String::from("F3 + B"),
        }
    }
}
