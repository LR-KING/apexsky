use apexsky::noobfstr as s;
use apexsky::pb::apexlegends::PlayerState;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::asset::Blob;
use super::MyOverlayState;

// A simple system to handle some keyboard input and toggle on/off the hittest.
pub fn toggle_mouse_passthrough(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();
    window.cursor.hit_test = keyboard_input.pressed(KeyCode::Insert);
}

pub fn ui_system(
    mut contexts: EguiContexts,
    mut overlay_state: ResMut<MyOverlayState>,
    diagnostics: Res<DiagnosticsStore>,
    blobs: Res<Assets<Blob>>,
) {
    use egui::{pos2, CentralPanel, Color32, ScrollArea};
    let ctx = contexts.ctx_mut();

    if !overlay_state.font_loaded {
        if let Some(font_blob) = blobs.get(&overlay_state.font_blob) {
            let mut egui_fonts = egui::FontDefinitions::default();
            egui_fonts.font_data.insert(
                "my_font".to_owned(),
                egui::FontData::from_owned(font_blob.bytes.to_owned()),
            );
            egui_fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "my_font".to_owned());
            egui_fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("my_font".to_owned());
            ctx.set_fonts(egui_fonts);

            overlay_state.font_loaded = true;
        }
    }

    struct DialogEsp {
        overlay_fps: String,
        game_fps: String,
        local_position: String,
        local_angles: String,
        aim_position: String,
        spectator_name: Vec<String>,
        allied_spectator_name: Vec<String>,
        teammates_info: Vec<PlayerState>,
    }

    let dialog_esp = {
        let state = overlay_state.shared_state.read();
        DialogEsp {
            overlay_fps: {
                // try to get a "smoothed" FPS value from Bevy
                if let Some(value) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                {
                    format!("{:.1}", value)
                } else {
                    s!("N/A").to_string()
                }
            },
            game_fps: format!("{:.1}", state.game_fps),
            local_position: state
                .local_player
                .as_ref()
                .and_then(|p| p.get_buf().origin.clone())
                .map(|pos| {
                    format!(
                        "{}{:.0}{}{:.0}{}{:.0}",
                        s!("x="),
                        pos.x,
                        s!(",y="),
                        pos.y,
                        s!(",z="),
                        pos.z
                    )
                })
                .unwrap_or_default(),
            local_angles: state
                .local_player
                .as_ref()
                .and_then(|p| p.get_buf().view_angles.clone())
                .map(|angle| {
                    format!(
                        "{}{:.2}{}{:.2}{}{:.2}",
                        s!("pitch="),
                        angle.x,
                        s!(",yew="),
                        angle.y,
                        s!(",roll="),
                        angle.z
                    )
                })
                .unwrap_or_default(),
            aim_position: format!(
                "{}{:.2}{}{:.2}{}{:.2}{}",
                s!("aim["),
                state.aim_target[0],
                s!(","),
                state.aim_target[1],
                s!(","),
                state.aim_target[2],
                s!("]")
            ),
            spectator_name: state.spectator_name.clone(),
            allied_spectator_name: state.allied_spectator_name.clone(),
            teammates_info: state.teammates.clone(),
        }
    };

    egui::Window::new(s!("Hello, world!"))
        .auto_sized()
        .default_pos(pos2(1600.0, 320.0))
        .show(ctx, |ui| {
            ui.label(format!(
                "{}{}{}{}{}",
                s!("Overlay("),
                dialog_esp.overlay_fps,
                s!(" FPS) Game("),
                dialog_esp.game_fps,
                s!(" FPS)")
            ));
            ui.add_space(5.0);
            ui.label(dialog_esp.local_position);
            ui.label(dialog_esp.local_angles);
            ui.label(dialog_esp.aim_position);

            ui.add_space(10.0);

            ScrollArea::vertical()
                .max_width(320.0)
                .max_height(480.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(s!("Teammates"));
                    });

                    if dialog_esp.teammates_info.is_empty() {
                        ui.label(s!("no teammates"));
                    }

                    for teammate in dialog_esp.teammates_info.iter() {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                            let name = teammate.player_name.to_owned();
                            ui.label(format!("{} - ", teammate.team_member_index));
                            ui.label(if dialog_esp.allied_spectator_name.contains(&name) {
                                egui::RichText::new(name).strong().color(Color32::GREEN)
                            } else {
                                egui::RichText::new(name).strong()
                            });
                            ui.add_space(5.0);
                            ui.label(teammate.damage_dealt.to_string());
                            ui.add_space(5.0);
                            ui.label(teammate.kills.to_string());
                        });
                    }
                });

            ui.add_space(10.0);

            ScrollArea::vertical()
                .max_width(320.0)
                .max_height(480.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(s!("Spectators"));
                    });

                    if dialog_esp.spectator_name.is_empty() {
                        ui.label(s!("no spectators"));
                    }

                    for name in dialog_esp.spectator_name.iter() {
                        ui.label(name);
                    }
                });
        });

    let panel_frame = egui::Frame {
        fill: Color32::TRANSPARENT, //ctx.style().visuals.window_fill(),
        rounding: 10.0.into(),
        stroke: egui::Stroke::NONE, //ctx.style().visuals.widgets.noninteractive.fg_stroke,
        outer_margin: 0.5.into(),   // so the stroke is within the bounds
        ..Default::default()
    };

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        info_bar_ui(ui, &overlay_state);
    });
}

fn info_bar_ui(ui: &mut egui::Ui, overlay_state: &MyOverlayState) {
    use egui::{pos2, Color32, Pos2, Rect, RichText};

    #[derive(Debug, Clone)]
    struct EspInfo {
        aimbot_fov: f32,
        aimbot_status_text: String,
        aimbot_status_color: Color32,
        spectators: usize,
        allied_spectators: usize,
    }

    let info = {
        let state = overlay_state.shared_state.read();
        if let Some(aimbot) = state.aimbot_state.as_ref() {
            let aimbot_mode = aimbot.get_settings().aim_mode;
            let (aimbot_status_color, aimbot_status_text) = if aimbot.is_locked() {
                (
                    if aimbot.get_gun_safety() {
                        Color32::GREEN
                    } else {
                        Color32::from_rgb(255, 165, 0) // Orange
                    },
                    s!("[TARGET LOCK!]").to_string(),
                )
            } else if aimbot.is_grenade() {
                (Color32::BLUE, s!("Skynade On").to_string())
            } else if aimbot_mode == 2 {
                (Color32::GREEN, s!("Aim On").to_string())
            } else if aimbot_mode == 0 {
                (Color32::RED, s!("Aim Off").to_string())
            } else {
                (Color32::RED, format!("{}{}", s!("Aim On "), aimbot_mode))
            };
            EspInfo {
                aimbot_fov: aimbot.get_max_fov(),
                aimbot_status_text,
                aimbot_status_color,
                spectators: state.spectator_name.len(),
                allied_spectators: state.allied_spectator_name.len(),
            }
        } else {
            EspInfo {
                aimbot_fov: 0.0,
                aimbot_status_text: s!("[Aimbot Offline]").to_string(),
                aimbot_status_color: Color32::GRAY,
                spectators: state.spectator_name.len(),
                allied_spectators: state.allied_spectator_name.len(),
            }
        }
    };

    // Draw rectangle
    let info_bar_rect = Rect::from_min_max(Pos2::ZERO, pos2(280.0, 30.0));
    ui.allocate_ui_at_rect(info_bar_rect, |ui| {
        let background_color = Color32::from_black_alpha((0.6 * 255.0 as f32).round() as u8);
        let rounding = 2.0;
        ui.painter()
            .rect_filled(info_bar_rect, rounding, background_color);

        // Draw red lines
        let line_color = Color32::from_rgb(255, 0, 0);
        let line_width = 2.0;
        ui.painter().line_segment(
            [
                pos2(info_bar_rect.min.x + rounding, info_bar_rect.min.y),
                pos2(info_bar_rect.max.x - rounding, info_bar_rect.min.y),
            ],
            (line_width, line_color),
        );
        ui.painter().line_segment(
            [
                pos2(
                    info_bar_rect.min.x + rounding,
                    info_bar_rect.max.y - line_width,
                ),
                pos2(
                    info_bar_rect.max.x - rounding,
                    info_bar_rect.max.y - line_width,
                ),
            ],
            (line_width, line_color),
        );

        // Draw text
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let text_size = 16.0;
            ui.add_space(rounding * 2.0);
            ui.label(
                RichText::new(format!("{}", info.spectators))
                    .color(if info.spectators == 0 {
                        Color32::GREEN
                    } else {
                        Color32::RED
                    })
                    .size(text_size),
            );
            ui.label(RichText::new(s!("--")).size(text_size));
            ui.label(
                RichText::new(format!("{}", info.allied_spectators))
                    .color(Color32::GREEN)
                    .size(text_size),
            );
            ui.label(RichText::new(s!("--")).size(text_size));
            ui.label(
                RichText::new(format!("{:.1}", info.aimbot_fov))
                    .color(Color32::WHITE)
                    .size(text_size),
            );
            ui.label(RichText::new(s!("--")).size(text_size));
            ui.label(
                RichText::new(info.aimbot_status_text)
                    .color(info.aimbot_status_color)
                    .size(text_size),
            );
            ui.add_space(rounding * 2.0);
        });
    });
}

// // info ui
// commands
//     .spawn(NodeBundle {
//         style: Style {
//             position_type: PositionType::Absolute,
//             left: Val::Px(0.0),
//             top: Val::Px(0.0),
//             width: Val::Px(280.0),
//             height: Val::Px(30.0),
//             ..Default::default()
//         },
//         background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.6)),
//         ..Default::default()
//     })
//     .with_children(|parent| {
//         let font_handle = asset_server.load(s!("fonts/LXGWNeoXiHei.ttf").to_string());
//         parent.spawn(
//             TextBundle::from_sections(vec![
//                 TextSection {
//                     value: "Hello, ".to_string(),
//                     style: TextStyle {
//                         font: font_handle.clone(),
//                         font_size: 16.0,
//                         color: Color::rgb(1.0, 0.0, 0.0),
//                     },
//                 },
//                 TextSection {
//                     value: "Bevy".to_string(),
//                     style: TextStyle {
//                         font: font_handle.clone(),
//                         font_size: 16.0,
//                         color: Color::rgb(0.0, 1.0, 0.0),
//                     },
//                 },
//                 TextSection {
//                     value: "!".to_string(),
//                     style: TextStyle {
//                         font: font_handle.clone(),
//                         font_size: 16.0,
//                         color: Color::rgb(0.0, 0.0, 1.0),
//                     },
//                 },
//             ])
//             .with_style(Style {
//                 align_self: AlignSelf::Center,
//                 margin: UiRect {
//                     left: Val::Px(4.),
//                     right: Val::Px(4.),
//                     ..Default::default()
//                 },
//                 ..Default::default()
//             }),
//         );
//     });
