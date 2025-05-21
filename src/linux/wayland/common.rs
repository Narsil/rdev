use serde::Deserialize;
use std::process::Command;

pub struct Display {}

#[derive(Debug, Deserialize)]
struct SwayDisplay {
    rect: Rect,
}

#[derive(Debug, Deserialize)]
struct HyprDisplay {
    #[serde(rename = "activeWorkspace")]
    active_workspace: Rect,
}

#[derive(Debug, Deserialize)]
struct Rect {
    width: usize,
    height: usize,
}

fn get_sway_size() -> Option<(usize, usize)> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_outputs", "-r"])
        .output()
        .ok()?
        .stdout;

    let displays: Vec<SwayDisplay> = serde_json::from_slice(&output).ok()?;
    let rect = &displays.get(0)?.rect;
    Some((rect.width, rect.height))
}

fn get_hyprland_size() -> Option<(usize, usize)> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()?
        .stdout;

    let displays: Vec<HyprDisplay> = serde_json::from_slice(&output).ok()?;
    let rect = &displays.get(0)?.active_workspace;
    Some((rect.width, rect.height))
}

impl Display {
    pub fn new() -> Option<Self> {
        Some(Self {})
    }

    pub fn get_size(&self) -> Option<(usize, usize)> {
        if let Some(sway) = get_sway_size() {
            Some(sway)
        } else if let Some(hyprland) = get_hyprland_size() {
            Some(hyprland)
        } else {
            None
        }
    }

    pub fn get_mouse_pos(&self) -> Option<(usize, usize)> {
        Some((0, 0))
    }
}
