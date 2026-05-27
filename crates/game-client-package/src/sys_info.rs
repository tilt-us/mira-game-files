use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::Paragraph;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::{Pid, ProcessesToUpdate, System, get_current_pid};

pub struct DebugScreenUiPlugin;

impl Plugin for DebugScreenUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugScreenState::new());
        app.add_systems(Update, update_debug_screen_metrics);
    }
}

#[derive(Resource)]
struct DebugScreenState {
    system: System,
    app_pid: Option<Pid>,
    update_timer: Timer,
    fps_smoothed: f32,
    cpu_name: String,
    gpu_name: String,
    gpu_vram_used_mib: Option<u64>,
    gpu_vram_total_mib: Option<u64>,
    gpu_load_percent: Option<f32>,
}

impl DebugScreenState {
    fn new() -> Self {
        let mut system = System::new_all();
        let app_pid = get_current_pid().ok();

        system.refresh_cpu_usage();
        if let Some(pid) = app_pid {
            system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        }

        let cpu_name = system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        Self {
            system,
            app_pid,
            update_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            fps_smoothed: 0.0,
            cpu_name,
            gpu_name: "Unknown GPU".to_string(),
            gpu_vram_used_mib: None,
            gpu_vram_total_mib: None,
            gpu_load_percent: None,
        }
    }
}

fn update_debug_screen_metrics(
    time: Res<Time>,
    adapter_info: Option<Res<RenderAdapterInfo>>,
    mut state: ResMut<DebugScreenState>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
    all_entities: Query<Entity>,
) {
    let delta = time.delta_secs();
    if delta > 0.0 {
        let current_fps = 1.0 / delta;
        state.fps_smoothed = if state.fps_smoothed <= 0.0 {
            current_fps
        } else {
            state.fps_smoothed * 0.9 + current_fps * 0.1
        };
    }

    if !state.update_timer.tick(time.delta()).just_finished() {
        return;
    }

    state.system.refresh_cpu_usage();

    let mut app_cpu_usage_total = 0.0_f32;
    let mut app_memory_bytes = None;
    if let Some(pid) = state.app_pid {
        state
            .system
            .refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        if let Some(process) = state.system.process(pid) {
            // sysinfo process CPU usage is cumulative over all logical cores:
            // 100% per core, e.g. 10 cores => max 1000%.
            app_cpu_usage_total = process.cpu_usage();
            app_memory_bytes = Some(process.memory());
        }
    }

    let system_cpu_usage = state.system.global_cpu_usage();
    let total_cores = state.system.cpus().len();
    let app_cpu_pool_percent = if total_cores > 0 {
        app_cpu_usage_total / total_cores as f32
    } else {
        0.0
    };
    let app_cores_used = app_cpu_usage_total / 100.0;
    let app_cores_used_whole = app_cores_used.round() as u32;
    let sample_window_ms = state.update_timer.duration().as_secs_f32() * 1000.0;
    let app_core_time_ms = app_cores_used * sample_window_ms;
    let entities_count = all_entities.iter().count();
    let app_ram_text = app_memory_bytes
        .map(format_bytes_as_ram)
        .unwrap_or_else(|| "N/A".to_string());

    if let Some(adapter_info) = adapter_info
        && !adapter_info.name.trim().is_empty()
    {
        state.gpu_name = adapter_info.name.clone();
    }

    if let Some((gpu_name, gpu_vram_used_mib, gpu_vram_total_mib, gpu_load_percent)) =
        query_nvidia_smi()
    {
        state.gpu_name = gpu_name;
        state.gpu_vram_used_mib = Some(gpu_vram_used_mib);
        state.gpu_vram_total_mib = Some(gpu_vram_total_mib);
        state.gpu_load_percent = Some(gpu_load_percent);
    } else if let Some((gpu_vram_used_mib, gpu_vram_total_mib, gpu_load_percent)) =
        query_amd_sysfs_metrics()
    {
        state.gpu_vram_used_mib = Some(gpu_vram_used_mib);
        state.gpu_vram_total_mib = Some(gpu_vram_total_mib);
        state.gpu_load_percent = Some(gpu_load_percent);
    }

    set_paragraph(
        &mut paragraph_q,
        "debug-fps-value",
        format!("{:.1}", state.fps_smoothed),
    );
    set_paragraph(&mut paragraph_q, "debug-cpu-name-value", state.cpu_name.clone());
    set_paragraph(
        &mut paragraph_q,
        "debug-cpu-load-value",
        format!("{:.1}% / {:.1}%", app_cpu_pool_percent, system_cpu_usage),
    );
    set_paragraph(
        &mut paragraph_q,
        "debug-ram-value",
        app_ram_text,
    );
    set_paragraph(&mut paragraph_q, "debug-gpu-name-value", state.gpu_name.clone());
    set_paragraph(
        &mut paragraph_q,
        "debug-gpu-vram-value",
        match (state.gpu_vram_used_mib, state.gpu_vram_total_mib) {
            (Some(current), Some(max)) => {
                format!(
                    "{} / {}",
                    format_mib_as_ram(current),
                    format_mib_as_ram(max)
                )
            }
            _ => "N/A".to_string(),
        },
    );
    set_paragraph(
        &mut paragraph_q,
        "debug-gpu-load-value",
        match state.gpu_load_percent {
            Some(load) => format!("{load:.1}%"),
            None => "N/A".to_string(),
        },
    );
    set_paragraph(
        &mut paragraph_q,
        "debug-cores-value",
        format!("{app_cores_used_whole} / {total_cores}"),
    );
    set_paragraph(
        &mut paragraph_q,
        "debug-core-time-value",
        format!("{app_core_time_ms:.1} ms"),
    );
    set_paragraph(
        &mut paragraph_q,
        "debug-entities-value",
        entities_count.to_string(),
    );
}

fn set_paragraph(
    paragraph_q: &mut Query<(&mut Paragraph, &CssID), With<Paragraph>>,
    target_id: &str,
    value: String,
) {
    for (mut paragraph, css_id) in paragraph_q.iter_mut() {
        if css_id.0 == target_id {
            paragraph.text = value;
            return;
        }
    }
}

fn query_nvidia_smi() -> Option<(String, u64, u64, f32)> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.used,memory.total,utilization.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8(output.stdout).ok()?;
    let line = raw.lines().find(|line| !line.trim().is_empty())?;
    let mut parts = line.split(',').map(str::trim);

    let gpu_name = parts.next()?.to_string();
    let gpu_vram_used_mib = parts.next()?.parse::<u64>().ok()?;
    let gpu_vram_total_mib = parts.next()?.parse::<u64>().ok()?;
    let gpu_load_percent = parts.next()?.parse::<f32>().ok()?;

    Some((
        gpu_name,
        gpu_vram_used_mib,
        gpu_vram_total_mib,
        gpu_load_percent,
    ))
}

fn query_amd_sysfs_metrics() -> Option<(u64, u64, f32)> {
    let card_device = find_amd_card_device_path()?;

    let vram_used_bytes = read_u64_from_file(card_device.join("mem_info_vram_used"))?;
    let vram_total_bytes = read_u64_from_file(card_device.join("mem_info_vram_total"))?;
    let gpu_busy_percent = read_f32_from_file(card_device.join("gpu_busy_percent"))?;
    let gpu_vram_used_mib = vram_used_bytes / (1024 * 1024);
    let gpu_vram_total_mib = vram_total_bytes / (1024 * 1024);

    Some((gpu_vram_used_mib, gpu_vram_total_mib, gpu_busy_percent))
}

fn find_amd_card_device_path() -> Option<PathBuf> {
    let drm_root = Path::new("/sys/class/drm");
    let entries = fs::read_dir(drm_root).ok()?;

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_str()?;
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }

        let card_path = entry.path();
        let device_path = card_path.join("device");
        let vendor = read_trimmed_file(device_path.join("vendor"))?;
        if vendor.eq_ignore_ascii_case("0x1002") {
            return Some(device_path);
        }
    }

    None
}

fn read_trimmed_file(path: PathBuf) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    Some(raw.trim().to_string())
}

fn read_u64_from_file(path: PathBuf) -> Option<u64> {
    read_trimmed_file(path)?.parse::<u64>().ok()
}

fn read_f32_from_file(path: PathBuf) -> Option<f32> {
    read_trimmed_file(path)?.parse::<f32>().ok()
}

fn format_bytes_as_ram(memory_bytes: u64) -> String {
    let mb = memory_bytes as f64 / 1_000_000.0;
    if mb > 1000.0 {
        format!("{:.2} GB", mb / 1000.0)
    } else {
        format!("{:.0} MB", mb)
    }
}

fn format_mib_as_ram(mib: u64) -> String {
    let mb = mib as f64 * 1.048_576;
    if mb > 1000.0 {
        format!("{:.2} GB", mb / 1000.0)
    } else {
        format!("{:.0} MB", mb)
    }
}
