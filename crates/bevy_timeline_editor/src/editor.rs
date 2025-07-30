use bevy::app::{App, PreUpdate, Startup, Update};
use bevy::DefaultPlugins;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_egui::egui::{self, Frame, Pos2, Shape, Stroke, Color32, Rect, Vec2, FontId, Widget, Sense, PointerButton, PointerState, Id, Rangef, Align, StrokeKind, ScrollArea, Slider, UiBuilder, Ui, InnerResponse};
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_timeline_runtime::prelude::{TimelineAnimation, TimelineAnimationSet};

fn format_f32(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{:.0}", value)
    } else if (value * 10.0).fract() == 0.0 {
        format!("{:.1}", value)
    } else {
        format!("{:.2}", value)
    }
}

pub struct TimelineEditorSettings {
    min_msec_width:f32,
    default_target_name_width:f32,
    max_target_name_width:f32,

    ruler_height:f32,
    target_height:f32,

    key_size:f32,

    min_zoom:f32,
    max_zoom:f32,

    time_label_size:f32,

    focus_time_pad_x:f32,
    focus_time_pad_y:f32,
}

impl Default for TimelineEditorSettings {
    fn default() -> Self {
        Self {
            min_msec_width:1.0,
            default_target_name_width: 100.,
            max_target_name_width: 300.,

            ruler_height: 30.,
            target_height: 20.,

            key_size: 4.,

            min_zoom: 3.0,
            max_zoom: 15.,

            time_label_size: 10.,

            focus_time_pad_x: 6.,
            focus_time_pad_y: 2.,
        }
    }
}

pub struct TargetState {
    selected_keyframes : Vec<usize>,
}


#[derive(Resource)]
pub struct TimelineEditor {
    ui_settings: TimelineEditorSettings,
    time: f32,
    zoom: f32,
    scroll_offset_x: f32,
    scroll_offset_y: f32,
    scroll_delta : Option<f32>,
    selected_keyframes: Vec<(usize, f32)>, // (target_idx, time)
    scroll_start: Option<Pos2>,
    drag_start: Option<Pos2>,
    selected_rect: Option< (bool,Rect) >,
    target_name_width : f32,
}

impl Default for TimelineEditor {
    fn default() -> Self {
        let ui_settings = TimelineEditorSettings::default();
        let zoom = ui_settings.min_zoom;
        let target_name_width = ui_settings.default_target_name_width;
        Self {
            ui_settings,
            time: 0.,
            zoom,
            scroll_offset_x: 0.0,
            scroll_offset_y: 0.0,
            scroll_delta: None,
            selected_keyframes: Vec::new(),
            drag_start:None,
            selected_rect: None,
            scroll_start: None,
            target_name_width,
        }
    }
}

impl TimelineEditor {
    fn new() -> Self {
        Default::default()
    }

    pub fn ui(&mut self, ui:&mut Ui, anim_set:Vec<&TimelineAnimationSet>, assets:&mut ResMut<Assets<TimelineAnimation>> ) {
        let key_size = self.ui_settings.key_size;
        let max_target_name_width = self.ui_settings.max_target_name_width;
        let focus_time_pad_x = self.ui_settings.focus_time_pad_x;
        let focus_time_pad_y = self.ui_settings.focus_time_pad_y;
        let min_msec_width = self.ui_settings.min_msec_width;
        let min_zoom = self.ui_settings.min_zoom;
        let max_zoom = self.ui_settings.max_zoom;
        let ruler_height = self.ui_settings.ruler_height;
        let frame = Frame::new().fill(Color32::from_gray(20));
        frame.show(ui, |ui| {
            let ruler_response = ui.vertical( |ui| {
                // Ruler (시간 표시)
                let ruler_response = ui.horizontal( |ui| {
                    self.draw_ruler(ui)
                });

                // Scrollable area
                let mut scroll_response = egui::ScrollArea::vertical()
                    .enable_scrolling(false)
                    .auto_shrink( [false;2] )
                    .animated(false)
                    .show(ui, |ui| {
                        ui.scroll_with_delta( Vec2::new(0., self.scroll_delta.take().unwrap_or(0.) ) ); //항상 실행해야함. 이것을 호출해야 내장된 스크롤이 disable 되는 효과를 가져옴
                        let selected_rect = self.selected_rect.take();
                        //selector 를 키프레임 크기에 맞춰 재가공
                        let selected_rect = selected_rect.map( |(shift,mut rect)| {
                            rect.min.x -= key_size;
                            rect.min.y -= key_size;
                            rect.max.x += key_size;
                            rect.max.y += key_size;
                            (shift, rect)
                        });

                        // for anim_set in anim_set.iter() {
                        //     for anim_handle in anim_set.anim_handles.iter() {
                        //         if let Some(anim) = assets.get(anim_handle) {
                        //             println!("{}", anim.name);
                        //         }
                        //     }
                        // }

                        // let len = self.targets.len();
                        // for index in (0 .. len) {
                        //     self.draw_target(ui, index, &selected_rect);
                        // }
                    });

                self.scroll_offset_y = scroll_response.state.offset.y;
                ruler_response.inner
            }).inner;

            //타겟 넓이 조정 센서
            let sep_pos = Pos2::new( ruler_response.rect.min.x-5. , ruler_response.rect.min.y );
            let separator_rect = Rect::from_min_size(sep_pos, Vec2::new(5.0, ruler_response.rect.height() ) );
            let separator_response = ui.interact(separator_rect, Id::new("target_name_resizer"), Sense::drag() );
            if separator_response.hovered() {
                ui.output_mut( |o| o.cursor_icon = egui::CursorIcon::ResizeColumn );
            }
            if separator_response.dragged() {
                self.target_name_width += separator_response.drag_delta().x;
                self.target_name_width = self.target_name_width.clamp(50.0, max_target_name_width);  // 최소 및 최대 너비 설정
            }

            //타임커서 그리기와 컨트롤
            //1sec 길이
            let time_string = format_f32(self.time);
            let time_string_size = ui.fonts( |r| r.layout_no_wrap( time_string.clone(), FontId::default(), Color32::WHITE).size() );
            let time_label_size = time_string_size + Vec2::new(focus_time_pad_x*2., focus_time_pad_y*2.);
            let one_sec_width = self.zoom * min_msec_width * 10.;
            let focused_time_x = self.time * one_sec_width + ruler_response.rect.min.x - self.scroll_offset_x;
            let focus_label_rect = Rect::from_min_size( Pos2::new( focused_time_x , ui.min_rect().min.y ), time_label_size );
            let mut focus_time_rect = Rect::from_two_pos(ruler_response.rect.min, ui.max_rect().max);
            let painter = ui.painter().with_clip_rect( focus_time_rect );
            painter.rect_filled( focus_label_rect.clone(), 2., Color32::from_rgba_unmultiplied(61,101,161, 255 ) );
            painter.rect_stroke( focus_label_rect, 2., Stroke::new(1.0, Color32::from_rgba_unmultiplied(79,120,191, 255 )), StrokeKind::Inside );
            painter.vline( focused_time_x, Rangef::new(ui.min_rect().min.y+1.0 , ui.max_rect().max.y),
                           Stroke::new(3.0, Color32::from_rgba_unmultiplied(79,120,191, 255 )) );
            painter.text( Pos2::new( focused_time_x+focus_time_pad_x, ui.min_rect().min.y+focus_time_pad_y ), egui::Align2::LEFT_TOP, time_string, FontId::default(), Color32::WHITE );
            let response = ui.interact( ruler_response.rect, egui::Id::new("focus_time"), Sense::click_and_drag() );
            if response.clicked_by(PointerButton::Primary) || response.dragged_by(PointerButton::Primary) {
                if let Some(pos) = response.interact_pointer_pos() {
                    let time = (pos.x - ruler_response.rect.min.x + self.scroll_offset_x)  / one_sec_width;
                    let time = (time * 10.0).round() / 10.0; //첫째자리만 남김
                    self.time = time;
                }
            }

            //룰러+키프레임 영역에서 마우스 가운데 버튼 드래그 이동
            let mut rect = ui.max_rect();
            rect.min.x += self.target_name_width;
            ui.input( |cx| {
                let pointer = &cx.pointer;
                if let Some(pos) = pointer.interact_pos() {
                    if rect.contains( pos ) {
                        // 마우스 휠 감지
                        if cx.raw_scroll_delta.y != 0.0 {
                            if cx.raw_scroll_delta.y > 0. {
                                self.zoom += 1.0;
                            } else {
                                self.zoom -= 1.0;
                            }
                            self.zoom = self.zoom.clamp(min_zoom, max_zoom);
                        }

                        if pointer.middle_down() {
                            if let Some(_scroll_start) = self.scroll_start {
                                let drag_delta = pointer.delta();
                                self.scroll_offset_x -= drag_delta.x; // / self.zoom;
                                if self.scroll_offset_x < 0. {
                                    self.scroll_offset_x = 0.; //don't use clamp
                                }
                                if drag_delta.y != 0. {
                                    self.scroll_delta = Some( drag_delta.y );
                                }
                            } else {
                                self.scroll_start = Some(pointer.interact_pos().unwrap());
                            }
                        } else {
                            self.scroll_start = None;
                        }
                    }
                }
            });

            // 마우스 왼쪽 드래그로 선택 영역 표시
            rect.min.y += ruler_height;
            let response = ui.interact( rect, Id::new("key_selector"), Sense::click_and_drag() );
            if response.drag_started_by(PointerButton::Primary) {
                self.drag_start = response.interact_pointer_pos();
            } else if response.dragged_by(PointerButton::Primary) || response.drag_stopped_by(PointerButton::Primary) {
                if let Some(start) = self.drag_start {
                    if let Some(mut pos) = response.interact_pointer_pos() {
                        if pos.x < ui.min_rect().min.x + self.target_name_width {
                            pos.x = ui.min_rect().min.x + self.target_name_width;
                        }
                        if pos.y < ui.min_rect().min.y + ruler_height {
                            pos.y = ui.min_rect().min.y + ruler_height
                        }
                        let selector_rect = Rect::from_two_pos(start, pos);

                        ui.painter().rect_filled(
                            selector_rect,
                            0.0,
                            Color32::from_rgba_unmultiplied(100, 100, 255, 50),
                        );
                        ui.painter().rect_stroke(
                            selector_rect,
                            0.0,
                            Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 100, 255, 200)),
                            StrokeKind::Inside
                        );

                        if response.drag_stopped_by(PointerButton::Primary) {
                            let is_shift = ui.input( |cx| cx.modifiers.shift );
                            self.selected_rect = Some( (is_shift,selector_rect) );
                            self.drag_start = None;
                        }
                    }
                } // end of drag_start
            } else if response.clicked_by(PointerButton::Primary) {
                if let Some( pos) = response.interact_pointer_pos() {
                    let is_shift = ui.input( |cx| cx.modifiers.shift );
                    self.selected_rect = Some( (is_shift,Rect::from_min_size(pos, Vec2::ZERO)) );
                }
            }
        });
    }

    fn draw_ruler(&self, ui: &mut egui::Ui) -> egui::Response {
        let min_msec_width = self.ui_settings.min_msec_width;
        let time_label_size = self.ui_settings.time_label_size;
        let ruler_height = self.ui_settings.ruler_height;
        let fid = FontId::monospace(time_label_size);
        let sec_stroke = Stroke::new(1.0, Color32::WHITE);
        let mid_stroke = Stroke::new(0.7, Color32::from_white_alpha(120));
        let dot_stroke = Stroke::new(0.3, Color32::from_white_alpha(150));

        ui.allocate_exact_size( Vec2::new(self.target_name_width, ruler_height), Sense::hover() );
        //ui.add_space(self.target_name_width);
        let (response,painter) = ui.allocate_painter( Vec2::new(ui.available_width(), ruler_height), Sense::hover() );
        let mut offset = response.rect.min.to_vec2();

        //1msec 길이
        let one_msec_width = self.zoom * min_msec_width;

        //1sec 길이
        let one_sec_width = one_msec_width * 10.;

        //현재 횡스크롤된 width
        let scroll_offset = self.scroll_offset_x;
        let pad_scroll = self.scroll_offset_x % one_msec_width;

        let mut x = 0.;
        while x < response.rect.width() {
            let time_x = x + scroll_offset - (scroll_offset%one_msec_width); //? (self.zoom * scroll_offset)
            let (y_len, label, stroke):(Option<f32>, String, Stroke) = if time_x % one_sec_width == 0. {
                ( Some( 0. ), format!("{}", time_x/one_sec_width ), sec_stroke.clone() )
            } else if time_x % one_sec_width == one_sec_width/2. {
                ( Some( (ruler_height * 0.5) ), String::new(), mid_stroke.clone() )
            } else {
                if self.zoom > 3.0 {
                    ( Some(ruler_height * 0.75), String::new(), dot_stroke.clone() )
                } else {
                    ( None, String::new(), dot_stroke.clone() )
                }
            };

            if let Some(y_len) = y_len {
                painter.line_segment( [Pos2::new(x-pad_scroll, y_len)+offset, Pos2::new(x-pad_scroll, ruler_height)+offset], stroke );
                if !label.is_empty() {
                    painter.text(
                        Pos2::new(x+2.-pad_scroll  , 1.)+offset,
                        egui::Align2::LEFT_TOP,
                        label,
                        fid.clone(),
                        Color32::WHITE,
                    );
                }

            }
            x += one_msec_width;
        }
        response
    }

    fn draw_target(
        &mut self,
        ui: &mut egui::Ui,
        index:usize,
        selected_rect:&Option<(bool,Rect)>
    ) {
        // let key_size = self.ui_settings.key_size;
        // let min_msec_width = self.ui_settings.min_msec_width;
        // let target_height = self.ui_settings.target_height;
        // let target = &mut self.targets[index];
        // let stroke_normal = Stroke::new(1.0, Color32::GRAY);
        // let stroke_selected = Stroke::new(1.0, Color32::WHITE);
        // ui.horizontal(|ui| {
        //     ui.add_sized(Vec2::new(self.target_name_width, target_height), egui::Label::new(target.name.as_str()).selectable(false).truncate() );
        // 
        //     // 오른쪽: 키프레임 표시
        //     let (response,painter) = ui.allocate_painter( Vec2::new(ui.available_width(), target_height), Sense::hover() );
        //     if index % 2 == 0 {
        //         painter.rect_filled( response.rect, 0., Color32::from_rgba_unmultiplied(80, 80, 80, 50) );
        //     } else {
        //         painter.rect_filled( response.rect, 0., Color32::default() );
        //     }
        //     for (i,keyframe) in target.keyframes.iter_mut().enumerate() {
        //         let x = response.rect.min.x + (keyframe.time * self.zoom * min_msec_width*10.) - self.scroll_offset_x;
        //         let pos = Pos2::new(x, response.rect.min.y + response.rect.height()/2. );
        //         if let Some( (shift_pressed,rect) ) = selected_rect {
        //             if rect.contains( pos ) {
        //                 keyframe.selected = true;
        //             } else {
        //                 if !shift_pressed {
        //                     keyframe.selected = false;
        //                 }
        //             }
        //         }
        //         if x < response.rect.min.x {
        //             continue;
        //         }
        //         if x > response.rect.max.x {
        //             continue;
        //             //break; //for selected_rect
        //         }
        //         let stroke = if keyframe.selected {
        //             stroke_selected
        //         } else {
        //             stroke_normal
        //         };
        //         painter.circle_stroke( pos, key_size, stroke );
        //     }
        // 
        // 
        // });
    }


    fn select_keys(&mut self, shift:bool, rect:Rect) {

        // let circle_rect = Rect::from_center_size(pos, Vec2::new(KEY_SIZE * 2.0, KEY_SIZE * 2.0));
        //
        // let click_response = ui.interact(circle_rect, ui.id(), Sense::click());
        // ui.input( |cx| {
        //     if cx.modifiers.shift {
        //         keyframe.selected = !keyframe.selected;
        //     } else {
        //         single_click = Some( i );
        //     }
        // });
        //
        // // 마우스 입력 처리 (키프레임 선택/다중 선택 등)
        // ui.input( |cx| {
        //     // if pointer.any_click() && ui.rect_contains_pointer(rect) {
        //     //     if is_selected {
        //     //         self.selected_keyframes.retain(|&x| x != (target_idx, i));
        //     //     } else {
        //     //         self.selected_keyframes.push((target_idx, i));
        //     //     }
        //     // }
        //
        //     // if let Some(pointer_pos) = pointer.interact_pos() {
        //     //     // 클릭한 위치가 키프레임 근처인지 확인
        //     //     let distance = (pointer_pos - pos).length();
        //     //     if distance < 10.0 {
        //     //         if is_selected {
        //     //             // 이미 선택된 경우, 선택 해제
        //     //             // self.selected_keyframes.retain(|&x| x != (target_idx, i));
        //     //         } else {
        //     //             // 선택되지 않은 경우, 선택
        //     //             // self.selected_keyframes.push((target_idx, i));
        //     //         }
        //     //     }
        //     // }
        // });
    }
}
