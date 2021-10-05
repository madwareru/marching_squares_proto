pub mod line_segment;
pub mod style;
pub mod draw_batcher;
pub mod parts;

use macroquad::prelude::*;
use nalgebra::{Vector2, Vector3};
use draw_batcher::BufferedDrawBatcher;
use crate::poly_line_2d::style::{JointStyle, EndCapStyle, LineStripStyle};
use crate::poly_line_2d::parts::{CapSegment, VSegments, SegmentTriangulation, DoubleCapSegment};

pub fn cross(lhs: Vector2<f32>, rhs: Vector2<f32>) -> f32 {
    let lhs: Vector3<f32> = [lhs.x, lhs.y, 0.0].into();
    let rhs: Vector3<f32> = [rhs.x, lhs.y, 0.0].into();
    let cross = lhs.cross(&rhs);
    cross.z
}

pub struct Painter {
    bezier_strip_buffer: Vec<[f32; 2]>,
    line_strip_buffer: Vec<[f32; 2]>,
    draw_batcher: BufferedDrawBatcher
}

impl Painter {
    pub fn new() -> Self {
        Self {
            bezier_strip_buffer: Vec::new(),
            line_strip_buffer: Vec::new(),
            draw_batcher: BufferedDrawBatcher::new()
        }
    }

    pub fn draw_circle(
        &mut self,
        center: [f32; 2],
        radius: f32,
        color: Color,
        num_subdivs: usize
    ) {
        if num_subdivs < 3 {
            return;
        }
        self.draw_batcher.clear_buffers();
        let theta_delta = (360.0 / num_subdivs as f32).to_radians();
        for id in 0..num_subdivs {
            let theta = id as f32 * theta_delta;
            let (cs, sn) = (theta.cos(), theta.sin());

            let left_2 = ((id + 1) * 2) as u16;
            let left_1 = left_2 - 1;

            let right_2 = if id < num_subdivs - 1 { left_2 + 2 } else { 2 };
            let right_1 = right_2 - 1;

            if id == 0 {
                self.draw_batcher.extend(
                    [
                        Vertex::new(
                            center[0], center[1], 0.0,
                            0.0, 0.0,
                            color
                        ),
                        Vertex::new(
                            center[0] + cs * radius,
                            center[1] + sn * radius,
                            0.0,
                            0.0, 0.0,
                            color
                        ),
                        Vertex::new(
                            center[0] + cs * (radius + 1.0),
                            center[1] + sn * (radius + 1.0),
                            0.0,
                            0.0, 0.0,
                            Color::new(color.r, color.g, color.b, 0.0)
                        )
                    ].iter().map(|it| *it),
                    [
                        0, left_1, right_1,
                        left_1, left_2, right_2,
                        left_1, right_2, right_1
                    ].iter().map(|it| *it)
                )
            } else {
                self.draw_batcher.extend(
                    [
                        Vertex::new(
                            center[0] + cs * radius,
                            center[1] + sn * radius,
                            0.0,
                            0.0, 0.0,
                            color
                        ),
                        Vertex::new(
                            center[0] + cs * (radius + 1.0),
                            center[1] + sn * (radius + 1.0),
                            0.0,
                            0.0, 0.0,
                            Color::new(color.r, color.g, color.b, 0.0)
                        )
                    ].iter().map(|it| *it),
                    [
                        0, left_1, right_1,
                        left_1, left_2, right_2,
                        left_1, right_2, right_1
                    ].iter().map(|it| *it)
                )
            }
        }
        self.draw_batcher.renderize(None);
    }

    pub fn draw_square_bezier_strip(
        &mut self,
        color: Color,
        thickness: f32,
        points: &[[f32; 2]]
    ) {
        self.draw_square_bezier_strip_ex(color, thickness, points.iter().map(|it| *it));
    }

    pub fn draw_square_bezier_strip_ex(
        &mut self,
        color: Color,
        thickness: f32,
        points: impl Iterator<Item = [f32; 2]>
    ) {
        self.bezier_strip_buffer.clear();
        self.bezier_strip_buffer.extend(points);

        let mut offset = 0;
        loop {
            if offset + 2 >= self.bezier_strip_buffer.len() {
                break;
            }
            let a = self.bezier_strip_buffer[offset];
            let control = self.bezier_strip_buffer[offset + 1];
            let b = self.bezier_strip_buffer[offset + 2];

            self.draw_square_bezier(a, b, control, color, thickness, 15);

            offset += 2;
        }
    }

    fn draw_square_bezier(
        &mut self,
        a: [f32; 2],
        b: [f32; 2],
        control: [f32; 2],
        color: Color,
        thickness: f32,
        num_subdivs: usize
    ) {
        let dt = 1.0 / num_subdivs as f32;
        self.draw_lines_ex(
            JointStyle::Bevel,
            EndCapStyle::Square,
            LineStripStyle::Open,
            color,
            thickness,
            (0..num_subdivs+1)
                .map(|ix|{
                    let t = (ix as f32 * dt).clamp(0.0, 1.0);
                    let t_opposite = 1.0 - t;

                    let xx1 = a[0] * t_opposite + control[0] * t;
                    let xx2 = b[0] * t + control[0] * t_opposite;

                    let yy1 = a[1] * t_opposite + control[1] * t;
                    let yy2 = b[1] * t + control[1] * t_opposite;

                    [
                        xx1 * t_opposite + xx2 * t,
                        yy1 * t_opposite + yy2 * t,
                    ]
                })
        );
    }

    pub fn draw_lines(
        &mut self,
        joint_style: JointStyle,
        end_cap_style: EndCapStyle,
        line_strip_style: LineStripStyle,
        color: Color,
        thickness: f32,
        points: &[[f32; 2]]
    ) {
        self.draw_lines_ex(
            joint_style,
            end_cap_style,
            line_strip_style,
            color,
            thickness,
            points.iter().map(|it| *it)
        )
    }

    pub fn draw_lines_ex(
        &mut self,
        joint_style: JointStyle,
        end_cap_style: EndCapStyle,
        line_strip_style: LineStripStyle,
        color: Color,
        thickness: f32,
        points: impl Iterator<Item = [f32; 2]>
    ) {
        self.line_strip_buffer.clear();
        self.line_strip_buffer.extend(points);

        let length = self.line_strip_buffer.len();
        if length <= 1 {
            return; // for lines we need at least two points
        }
        self.draw_batcher.clear_buffers();

        if length == 2 {
            let seg = DoubleCapSegment::new(
                [
                    self.line_strip_buffer[0][0],
                    self.line_strip_buffer[0][1]
                ].into(),
                [
                    self.line_strip_buffer[1][0],
                    self.line_strip_buffer[1][1]
                ].into(),
                thickness
            );
            seg.triangulate(0, color, end_cap_style)
                .extend_draw_batcher(&mut self.draw_batcher);
            self.draw_batcher.renderize(None);
            return;
        }

        match line_strip_style {
            LineStripStyle::Closed => {
                let last_id = length - 1;
                let a = [
                    (self.line_strip_buffer[0][0] + self.line_strip_buffer[1][0]) / 2.0,
                    (self.line_strip_buffer[0][1] + self.line_strip_buffer[1][1]) / 2.0
                ].into();
                let b = [
                    self.line_strip_buffer[0][0],
                    self.line_strip_buffer[0][1]
                ].into();
                let c = [
                    (self.line_strip_buffer[0][0] + self.line_strip_buffer[last_id][0]) / 2.0,
                    (self.line_strip_buffer[0][1] + self.line_strip_buffer[last_id][1]) / 2.0
                ].into();
                let v_segments = VSegments::new(a, b, c, thickness);
                let joint_style = match joint_style {
                    JointStyle::Miter if v_segments.angle_is_too_sharp() => JointStyle::Bevel,
                    _ => joint_style
                };
                v_segments.triangulate(0, color, joint_style)
                    .extend_draw_batcher(&mut self.draw_batcher);
                self.draw_batcher.renderize(None);

                let a = [
                    (self.line_strip_buffer[last_id-1][0] + self.line_strip_buffer[last_id][0]) / 2.0,
                    (self.line_strip_buffer[last_id-1][1] + self.line_strip_buffer[last_id][1]) / 2.0
                ].into();
                let b = [
                    self.line_strip_buffer[last_id][0],
                    self.line_strip_buffer[last_id][1]
                ].into();
                let c = [
                    (self.line_strip_buffer[0][0] + self.line_strip_buffer[last_id][0]) / 2.0,
                    (self.line_strip_buffer[0][1] + self.line_strip_buffer[last_id][1]) / 2.0
                ].into();
                let v_segments = VSegments::new(a, b, c, thickness);
                let joint_style = match joint_style {
                    JointStyle::Miter if v_segments.angle_is_too_sharp() => JointStyle::Bevel,
                    _ => joint_style
                };
                v_segments.triangulate(0, color, joint_style)
                    .extend_draw_batcher(&mut self.draw_batcher);
                self.draw_batcher.renderize(None);
            },
            LineStripStyle::Open => {
                let cap_segment_start = CapSegment::new(
                    [
                        (self.line_strip_buffer[0][0] + self.line_strip_buffer[1][0]) / 2.0,
                        (self.line_strip_buffer[0][1] + self.line_strip_buffer[1][1]) / 2.0
                    ].into(),
                    [
                        self.line_strip_buffer[0][0],
                        self.line_strip_buffer[0][1]
                    ].into(),
                    thickness
                );
                let cap_segment_end = CapSegment::new(
                    [
                        (self.line_strip_buffer[length-1][0] + self.line_strip_buffer[length-2][0]) / 2.0,
                        (self.line_strip_buffer[length-1][1] + self.line_strip_buffer[length-2][1]) / 2.0
                    ].into(),
                    [
                        self.line_strip_buffer[length-1][0],
                        self.line_strip_buffer[length-1][1]
                    ].into(),
                    thickness
                );
                cap_segment_start.triangulate(0, color, end_cap_style)
                    .extend_draw_batcher(&mut self.draw_batcher);
                self.draw_batcher.renderize(None);

                cap_segment_end.triangulate(0, color, end_cap_style)
                    .extend_draw_batcher(&mut self.draw_batcher);
                self.draw_batcher.renderize(None);
            }
        }

        for i in 0..length-2 {
            let a = [
                (self.line_strip_buffer[i][0] + self.line_strip_buffer[i + 1][0]) / 2.0,
                (self.line_strip_buffer[i][1] + self.line_strip_buffer[i + 1][1]) / 2.0
            ].into();
            let b = [
                self.line_strip_buffer[i + 1][0],
                self.line_strip_buffer[i + 1][1]
            ].into();
            let c = [
                (self.line_strip_buffer[i + 1][0] + self.line_strip_buffer[i + 2][0]) / 2.0,
                (self.line_strip_buffer[i + 1][1] + self.line_strip_buffer[i + 2][1]) / 2.0
            ].into();
            let v_segments = VSegments::new(a, b, c, thickness);

            let joint_style = match joint_style {
                JointStyle::Miter if v_segments.angle_is_too_sharp() => JointStyle::Bevel,
                _ => joint_style
            };

            let triangulation = v_segments.triangulate(0, color, joint_style);
            match triangulation {
                SegmentTriangulation::Straight { vertices, indices } => {
                    self.draw_batcher.extend(
                        vertices.iter().map(|it| Vertex::new(
                            it.pos_x, it.pos_y, 0.0,
                            0.0, 0.0,
                            Color::new(it.color.0, it.color.1, it.color.2, it.color.3)
                        )),
                        indices.iter().map(|it| *it)
                    );
                }
                SegmentTriangulation::Miter { vertices, indices } => {
                    self.draw_batcher.extend(
                        vertices.iter().map(|it| Vertex::new(
                            it.pos_x, it.pos_y, 0.0,
                            0.0, 0.0,
                            Color::new(it.color.0, it.color.1, it.color.2, it.color.3)
                        )),
                        indices.iter().map(|it| *it)
                    );
                }
                SegmentTriangulation::Bevel { vertices, indices } => {
                    self.draw_batcher.extend(
                        vertices.iter().map(|it| Vertex::new(
                            it.pos_x, it.pos_y, 0.0,
                            0.0, 0.0,
                            Color::new(it.color.0, it.color.1, it.color.2, it.color.3)
                        )),
                        indices.iter().map(|it| *it)
                    );
                }
            }
            self.draw_batcher.renderize(None);
        }

        self.draw_batcher.renderize(None);
    }
}