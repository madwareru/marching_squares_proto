pub mod poly_line_2d;

use macroquad::prelude::*;
use crate::poly_line_2d::Painter;
use crate::poly_line_2d::style::{JointStyle, EndCapStyle, LineStripStyle};

const TILE_SIZE: f32 = 96.0;
const TERRAIN_THRESHOLD: f32 = 0.001;

#[macroquad::main("marching_squares_proto")]
async fn main() {
    let mut painter = Painter::new();

    let mut camera_x = -TILE_SIZE / 4.0;
    let mut camera_y = -TILE_SIZE / 4.0;

    let mut camera_scale = 1.0;

    let inner_radius = TILE_SIZE / 4.0;
    let outer_radius = TILE_SIZE / 3.0;

    let mut screen_drag_state = None;

    let mut weights = [[0.0f32; 129]; 129];

    loop {
        clear_background(Color::new(0.03, 0.02, 0.05, 1.0));

        let (mouse_x, mouse_y) = mouse_position();
        let (_, mouse_wheel_y) = mouse_wheel();

        camera_scale = (camera_scale + mouse_wheel_y * get_frame_time()).clamp(0.1, 2.0);

        let local_tile_size = TILE_SIZE * camera_scale;

        if is_key_pressed(KeyCode::Space) && screen_drag_state.is_none() {
            screen_drag_state = Some((mouse_x / camera_scale, mouse_y / camera_scale, camera_x, camera_y))
        } else if is_key_released(KeyCode::Space) && screen_drag_state.is_some() {
            screen_drag_state = None;
        }

        match screen_drag_state {
            None => {}
            Some((drag_start_mx, drag_start_my, drag_start_cam_x, drag_start_cam_y)) => {
                camera_x = drag_start_cam_x - mouse_x / camera_scale + drag_start_mx;
                camera_y = drag_start_cam_y - mouse_y / camera_scale + drag_start_my;
            }
        }

        for j in 0..=128 {
            let coord_y = (j as f32 * TILE_SIZE - camera_y) * camera_scale;

            if coord_y < -2.0 {
                continue;
            }

            if coord_y > screen_height() {
                break;
            }

            for i in 0..=128 {
                let coord_x = (i as f32 * TILE_SIZE - camera_x) * camera_scale;

                if coord_x < -2.0 {
                    continue;
                }

                if coord_x > screen_width() {
                    break;
                }

                if is_mouse_button_down(MouseButton::Left) {
                    let dist = (
                        (coord_x - mouse_x) * (coord_x - mouse_x) +
                            (coord_y - mouse_y) * (coord_y - mouse_y)
                    ).sqrt();
                    let power = if dist > outer_radius {
                        0.0
                    } else if dist < inner_radius {
                        1.0
                    } else {
                        1.0 - (dist - inner_radius) / (outer_radius - inner_radius)
                    };
                    weights[j][i] = (weights[j][i] + 0.5 * get_frame_time() * power).clamp(0.0, 1.0);
                } else if is_mouse_button_down(MouseButton::Right) {
                    let dist = (
                        (coord_x - mouse_x)*(coord_x - mouse_x) +
                            (coord_y - mouse_y)*(coord_y - mouse_y)
                    ).sqrt();
                    let power = if dist > outer_radius {
                        0.0
                    } else if dist < inner_radius {
                        1.0
                    } else {
                        1.0 - (dist - inner_radius) / (outer_radius - inner_radius)
                    };
                    weights[j][i] = (weights[j][i] - 0.5 * get_frame_time() * power).clamp(0.0, 1.0);
                }

                let t = weights[j][i];
                let t_opposite = 1.0 - t;
                let color = Color::new(
                    0.2 * t_opposite + 0.1 * t,
                    0.2 * t_opposite + 1.0 * t,
                    0.2 * t_opposite + 0.7 * t,
                    1.0
                );

                draw_rectangle(coord_x - 4.0, coord_y - 4.0, 8.0, 8.0, color);

                if i > 0 && j > 0 {
                    let (
                        corner_nw, corner_ne,
                        corner_sw, corner_se
                    ) = (
                        weights[j - 1][i - 1], weights[j - 1][i],
                        weights[j][i - 1],     weights[j][i]
                    );
                    let right = if (corner_ne < TERRAIN_THRESHOLD && corner_se < TERRAIN_THRESHOLD) ||
                        (corner_ne >= TERRAIN_THRESHOLD && corner_se >= TERRAIN_THRESHOLD) {
                        None
                    } else {
                        if corner_ne >= TERRAIN_THRESHOLD {
                            Some([coord_x, coord_y - local_tile_size * (1.0 - corner_ne)])
                        } else {
                            Some([coord_x, coord_y - local_tile_size * (corner_se)])
                        }
                    };
                    let left = if (corner_nw < TERRAIN_THRESHOLD && corner_sw < TERRAIN_THRESHOLD) ||
                        (corner_nw >= TERRAIN_THRESHOLD && corner_sw >= TERRAIN_THRESHOLD) {
                        None
                    } else {
                        if corner_nw >= TERRAIN_THRESHOLD {
                            Some([coord_x - local_tile_size, coord_y - local_tile_size * (1.0 - corner_nw)])
                        } else {
                            Some([coord_x - local_tile_size, coord_y - local_tile_size * (corner_sw)])
                        }
                    };
                    let bottom = if (corner_sw < TERRAIN_THRESHOLD && corner_se < TERRAIN_THRESHOLD) ||
                        (corner_sw >= TERRAIN_THRESHOLD && corner_se >= TERRAIN_THRESHOLD) {
                        None
                    } else {
                        if corner_sw >= TERRAIN_THRESHOLD {
                            Some([coord_x - local_tile_size * (1.0 - corner_sw), coord_y])
                        } else {
                            Some([coord_x - local_tile_size * (corner_se), coord_y])
                        }
                    };
                    let top = if (corner_nw < TERRAIN_THRESHOLD && corner_ne < TERRAIN_THRESHOLD) ||
                        (corner_nw >= TERRAIN_THRESHOLD && corner_ne >= TERRAIN_THRESHOLD) {
                        None
                    } else {
                        if corner_nw >= TERRAIN_THRESHOLD {
                            Some([coord_x - local_tile_size * (1.0 - corner_nw), coord_y - local_tile_size])
                        } else {
                            Some([coord_x - local_tile_size * (corner_ne), coord_y - local_tile_size])
                        }
                    };
                    match (left, right, top, bottom) {
                        (Some(l), Some(r), None, None) => {
                            painter.draw_lines(
                                JointStyle::Miter,
                                EndCapStyle::Butt,
                                LineStripStyle::Open,
                                Color::new(0.1, 1.0, 0.7, 1.0),
                                2.0,
                                &[l, r]
                            );
                        },
                        (None, None, Some(t), Some(b)) => {
                            painter.draw_lines(
                                JointStyle::Miter,
                                EndCapStyle::Butt,
                                LineStripStyle::Open,
                                Color::new(0.1, 1.0, 0.7, 1.0),
                                2.0,
                                &[t, b]
                            );
                        },
                        (Some(l), None, Some(t), None) => {
                            painter.draw_lines(
                                JointStyle::Miter,
                                EndCapStyle::Butt,
                                LineStripStyle::Open,
                                Color::new(0.1, 1.0, 0.7, 1.0),
                                2.0,
                                &[l, t]
                            );
                        },
                        (None, Some(r), Some(t), None) => {
                            painter.draw_lines(
                                JointStyle::Miter,
                                EndCapStyle::Butt,
                                LineStripStyle::Open,
                                Color::new(0.1, 1.0, 0.7, 1.0),
                                2.0,
                                &[r, t]
                            );
                        },
                        (Some(l), None, None, Some(b)) => {
                            painter.draw_lines(
                                JointStyle::Miter,
                                EndCapStyle::Butt,
                                LineStripStyle::Open,
                                Color::new(0.1, 1.0, 0.7, 1.0),
                                2.0,
                                &[l, b]
                            );
                        },
                        (None, Some(r), None, Some(b)) => {
                            painter.draw_lines(
                                JointStyle::Miter,
                                EndCapStyle::Butt,
                                LineStripStyle::Open,
                                Color::new(0.1, 1.0, 0.7, 1.0),
                                2.0,
                                &[r, b]
                            );
                        },
                        (Some(l), Some(r), Some(t), Some(b)) if corner_nw >= TERRAIN_THRESHOLD && corner_se >= TERRAIN_THRESHOLD => {
                            painter.draw_lines(
                                JointStyle::Miter,
                                EndCapStyle::Butt,
                                LineStripStyle::Open,
                                Color::new(0.1, 1.0, 0.7, 1.0),
                                2.0,
                                &[r, b]
                            );
                            painter.draw_lines(
                                JointStyle::Miter,
                                EndCapStyle::Butt,
                                LineStripStyle::Open,
                                Color::new(0.1, 1.0, 0.7, 1.0),
                                2.0,
                                &[l, t]
                            );
                        },
                        (Some(l), Some(r), Some(t), Some(b)) if corner_ne >= TERRAIN_THRESHOLD && corner_sw >= TERRAIN_THRESHOLD => {
                            painter.draw_lines(
                                JointStyle::Miter,
                                EndCapStyle::Butt,
                                LineStripStyle::Open,
                                Color::new(0.1, 1.0, 0.7, 1.0),
                                2.0,
                                &[r, t]
                            );
                            painter.draw_lines(
                                JointStyle::Miter,
                                EndCapStyle::Butt,
                                LineStripStyle::Open,
                                Color::new(0.1, 1.0, 0.7, 1.0),
                                2.0,
                                &[l, b]
                            );
                        },
                        _ => {}
                    }
                }
            }
        }

        for i in (0..=128).step_by(8) {
            let color = if i % 16 != 0 {
                Color::new(
                    0.2 * 0.75 + 0.1 * 0.25,
                    0.2 * 0.75 + 0.7 * 0.25,
                    0.2 * 0.75 + 1.0 * 0.25,
                    0.5
                )
            } else {
                Color::new(
                    0.2 * 0.25 + 0.1 * 0.75,
                    0.2 * 0.25 + 0.7 * 0.75,
                    0.2 * 0.25 + 1.0 * 0.75,
                    0.5
                )
            };
            {
                let coord_x = -camera_x * camera_scale - 1.0;
                let coord_y = (i as f32 * TILE_SIZE - camera_y) * camera_scale - 1.0;
                draw_rectangle(
                    coord_x, coord_y,
                    128.0 * TILE_SIZE * camera_scale + 2.0,
                    2.0,
                    color
                );
            }
            {
                let coord_y = -camera_y * camera_scale - 1.0;
                let coord_x = (i as f32 * TILE_SIZE - camera_x) * camera_scale - 1.0;
                draw_rectangle(
                    coord_x, coord_y,
                    2.0,
                    128.0 * TILE_SIZE * camera_scale + 2.0,
                    color
                );
            }
        }

        painter.draw_lines_ex(
            JointStyle::Miter,
            EndCapStyle::Butt,
            LineStripStyle::Closed,
            WHITE,
            1.5,
            (0..72)
                .map(|degree| (degree as f32 * 5.0).to_radians())
                .map(|theta|
                    [
                        mouse_x + theta.cos() * inner_radius,
                        mouse_y + theta.sin() * inner_radius
                    ]
                )
        );

        painter.draw_lines_ex(
            JointStyle::Miter,
            EndCapStyle::Butt,
            LineStripStyle::Closed,
            GRAY,
            1.2,
            (0..72)
                .map(|degree| (degree as f32 * 5.0).to_radians())
                .map(|theta|
                    [
                        mouse_x + theta.cos() * outer_radius,
                        mouse_y + theta.sin() * outer_radius
                    ]
                )
        );

        next_frame().await;
    }
}