use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::test_utils::cwd_lock;
use game_client_package::config::{ClientConfigs, GeneralConfig, GraphicsConfig};

fn create_temp_test_dir() -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after unix epoch")
        .as_nanos();

    let dir = env::temp_dir().join(format!(
        "mira-game-testing-config-{}-{}",
        std::process::id(),
        now
    ));
    fs::create_dir_all(&dir).expect("failed to create temp test dir");
    dir
}

fn run_in_temp_dir<F>(test_fn: F)
where
    F: FnOnce(),
{
    struct TempDirCwdGuard {
        original_dir: PathBuf,
        temp_dir: PathBuf,
    }

    impl Drop for TempDirCwdGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original_dir);
            let _ = fs::remove_dir_all(&self.temp_dir);
        }
    }

    let _guard = cwd_lock().lock().expect("failed to acquire cwd lock");
    let original_dir = env::current_dir().expect("failed to read current dir");
    let temp_dir = create_temp_test_dir();
    let _temp_dir_guard = TempDirCwdGuard {
        original_dir,
        temp_dir: temp_dir.clone(),
    };

    env::set_current_dir(&temp_dir).expect("failed to switch to temp dir");
    test_fn();
}

#[test]
fn ensure_config_files_exists_creates_default_toml_files() {
    run_in_temp_dir(|| {
        ClientConfigs::ensure_config_files_exists();

        assert!(Path::new("config/client_general.toml").exists());
        assert!(Path::new("config/client_graphics.toml").exists());
        assert!(Path::new("config/client_input.toml").exists());

        let loaded = ClientConfigs::new();
        assert_eq!(loaded.config_general.language, "english");
        assert_eq!(loaded.config_graphics.window_width, 1270);
        assert_eq!(loaded.config_graphics.window_height, 720);
        assert_eq!(loaded.config_graphics.vsync, "default");
        assert_eq!(loaded.config_graphics.graphic_backend, "AUTO");
        assert_eq!(loaded.config_graphics.ui_scale, "3");
        assert!(!loaded.config_graphics.fullscreen);
    });
}

#[test]
fn save_all_persists_updated_values() {
    run_in_temp_dir(|| {
        let mut configs = ClientConfigs::default();
        configs.config_general.language = String::from("german");
        configs.config_graphics = GraphicsConfig {
            window_width: 1920,
            window_height: 1080,
            fullscreen: true,
            vsync: String::from("mailbox"),
            graphic_backend: String::from("VULKAN"),
            ui_scale: String::from("2"),
        };

        configs.save_all();

        let loaded = ClientConfigs::new();
        assert_eq!(loaded.config_general.language, "german");
        assert_eq!(loaded.config_graphics.window_width, 1920);
        assert_eq!(loaded.config_graphics.window_height, 1080);
        assert!(loaded.config_graphics.fullscreen);
        assert_eq!(loaded.config_graphics.vsync, "mailbox");
        assert_eq!(loaded.config_graphics.graphic_backend, "VULKAN");
        assert_eq!(loaded.config_graphics.ui_scale, "2");
    });
}

#[test]
fn load_reads_typed_config_file() {
    run_in_temp_dir(|| {
        ClientConfigs::ensure_config_files_exists();

        let general: GeneralConfig = ClientConfigs::load("config/client_general.toml");
        let graphics: GraphicsConfig = ClientConfigs::load("config/client_graphics.toml");

        assert_eq!(general.language, "english");
        assert_eq!(graphics.window_width, 1270);
        assert_eq!(graphics.window_height, 720);
    });
}
