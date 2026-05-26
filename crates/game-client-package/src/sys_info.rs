use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::Paragraph;
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
    gpu_vram_mib: Option<u64>,
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
            gpu_vram_mib: None,
            gpu_load_percent: None,
        }
    }
}

fn update_debug_screen_metrics(
    time: Res<Time>,
    adapter_info: Option<Res<RenderAdapterInfo>>,
    mut state: ResMut<DebugScreenState>,
    mut paragraph_q: Query<(&mut Paragraph, &CssID), With<Paragraph>>,
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

    let mut app_cpu_usage = 0.0_f32;
    if let Some(pid) = state.app_pid {
        state
            .system
            .refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        if let Some(process) = state.system.process(pid) {
            app_cpu_usage = process.cpu_usage();
        }
    }

    let system_cpu_usage = state.system.global_cpu_usage();

    if let Some(adapter_info) = adapter_info
        && !adapter_info.name.trim().is_empty()
    {
        state.gpu_name = adapter_info.name.clone();
    }

    if let Some((gpu_name, gpu_vram_mib, gpu_load_percent)) = query_nvidia_smi() {
        state.gpu_name = gpu_name;
        state.gpu_vram_mib = Some(gpu_vram_mib);
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
        format!("{:.1}% / {:.1}%", app_cpu_usage, system_cpu_usage),
    );
    set_paragraph(&mut paragraph_q, "debug-gpu-name-value", state.gpu_name.clone());
    set_paragraph(
        &mut paragraph_q,
        "debug-gpu-vram-value",
        match state.gpu_vram_mib {
            Some(vram) => format!("{vram} MiB"),
            None => "N/A".to_string(),
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

fn query_nvidia_smi() -> Option<(String, u64, f32)> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,utilization.gpu",
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
    let gpu_vram_mib = parts.next()?.parse::<u64>().ok()?;
    let gpu_load_percent = parts.next()?.parse::<f32>().ok()?;

    Some((gpu_name, gpu_vram_mib, gpu_load_percent))
}
