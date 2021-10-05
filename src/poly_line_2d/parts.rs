use super::line_segment::{PolySegment, LineSegment, Direction};
use super::{cross, style::JointStyle};
use nalgebra::Point2;
use macroquad::prelude::*;
use crate::poly_line_2d::style::EndCapStyle;
use crate::poly_line_2d::draw_batcher::BufferedDrawBatcher;

use indices_macro::make_indices;

pub(crate) struct VertexData {
    pub pos_x: f32,
    pub pos_y: f32,
    pub color: (f32, f32, f32, f32),
}

impl VertexData {
    pub fn new(
        pos_x: f32,
        pos_y: f32,
        color: (f32, f32, f32, f32),
    ) -> Self { Self { pos_x, pos_y, color } }
}

pub(crate) struct VSegments {
    lcs: PolySegment,
    rcs: PolySegment,
}

pub(crate) struct CapSegment(PolySegment, f32);

pub(crate) struct DoubleCapSegment(PolySegment, f32);

pub(crate) enum SegmentTriangulation {
    Straight { vertices: [VertexData; 8], indices: [u16; 30] },
    Miter { vertices: [VertexData; 12], indices: [u16; 36] },
    Bevel { vertices: [VertexData; 14], indices: [u16; 45] },
}

impl SegmentTriangulation {
    pub(crate) fn extend_draw_batcher(&self, draw_batcher: &mut BufferedDrawBatcher) {
        match self {
            SegmentTriangulation::Straight { vertices, indices } => {
                draw_batcher.extend(
                    vertices.iter().map(|it| Vertex::new(
                        it.pos_x, it.pos_y, 0.0,
                        0.0, 0.0,
                        Color::new(it.color.0, it.color.1, it.color.2, it.color.3),
                    )),
                    indices.iter().map(|it| *it),
                );
            }
            SegmentTriangulation::Miter { vertices, indices } => {
                draw_batcher.extend(
                    vertices.iter().map(|it| Vertex::new(
                        it.pos_x, it.pos_y, 0.0,
                        0.0, 0.0,
                        Color::new(it.color.0, it.color.1, it.color.2, it.color.3),
                    )),
                    indices.iter().map(|it| *it),
                );
            }
            SegmentTriangulation::Bevel { vertices, indices } => {
                draw_batcher.extend(
                    vertices.iter().map(|it| Vertex::new(
                        it.pos_x, it.pos_y, 0.0,
                        0.0, 0.0,
                        Color::new(it.color.0, it.color.1, it.color.2, it.color.3),
                    )),
                    indices.iter().map(|it| *it),
                );
            }
        }
    }
}

impl VSegments {
    pub(crate) fn new(a: Point2<f32>, b: Point2<f32>, c: Point2<f32>, thickness: f32) -> Self {
        let line_segment_first = LineSegment {
            a: a.clone(),
            b: b.clone(),
        };
        let line_segment_second = LineSegment {
            a: b.clone(),
            b: c.clone(),
        };
        Self {
            lcs: PolySegment::new(&line_segment_first, thickness),
            rcs: PolySegment::new(&line_segment_second, thickness),
        }
    }

    fn is_clockwise(&self) -> bool {
        let dir1 = self.lcs.c.direction(Direction::Normalized);
        let dir2 = self.rcs.c.direction(Direction::Normalized);
        cross(-dir1, dir2) > 0.0
    }

    pub(crate) fn angle_is_too_sharp(&self) -> bool {
        let dir1 = self.lcs.c.direction(Direction::Normalized);
        let dir2 = self.rcs.c.direction(Direction::Normalized);
        dir1.dot(&dir2) < -0.8
    }

    fn get_intersection(&self) -> Option<(Point2<f32>, Point2<f32>, Point2<f32>, Point2<f32>)> {
        let upper = self.lcs.u
            .intersection(
                self.rcs.u.clone()
            )?;
        let upper_aa = self.lcs.u_aa
            .intersection(
                self.rcs.u_aa.clone()
            )?;
        let lower = self.lcs.l
            .intersection(
                self.rcs.l.clone()
            )?;
        let lower_aa = self.lcs.l_aa
            .intersection(
                self.rcs.l_aa.clone()
            )?;
        Some((upper, upper_aa, lower, lower_aa))
    }

    pub(crate) fn triangulate(&self, start_id: u16, color: Color, style: JointStyle) -> SegmentTriangulation {
        let transparent_color = (color.r, color.g, color.b, color.a);
        let color = (color.r, color.g, color.b, color.a);
        match self.get_intersection() {
            None => SegmentTriangulation::Straight {
                vertices:
                [
                    VertexData::new(self.lcs.u_aa.a.x, self.lcs.u_aa.a.y, transparent_color),
                    VertexData::new(self.lcs.u.a.x, self.lcs.u.a.y, color),
                    VertexData::new(self.lcs.l.a.x, self.lcs.l.a.y, color),
                    VertexData::new(self.lcs.l_aa.a.x, self.lcs.l_aa.a.y, transparent_color),
                    VertexData::new(self.rcs.u_aa.b.x, self.rcs.u_aa.b.y, transparent_color),
                    VertexData::new(self.rcs.u.b.x, self.rcs.u.b.y, color),
                    VertexData::new(self.rcs.l.b.x, self.rcs.l.b.y, color),
                    VertexData::new(self.rcs.l_aa.b.x, self.rcs.l_aa.b.y, transparent_color)
                ],
                indices: make_indices! {
                    * 0 4 1 5;
                    * 1 5 2 6;
                    * 2 6 3 7;
                    * 0 1 2 3;
                    * 4 5 6 7;
                },
            },
            Some((upper, upper_aa, lower, lower_aa)) => {
                let is_clockwise = self.is_clockwise();
                match (is_clockwise, style) {
                    (false, JointStyle::Bevel) => {
                        SegmentTriangulation::Bevel {
                            vertices: [
                                VertexData::new(self.lcs.u_aa.a.x, self.lcs.u_aa.a.y, transparent_color),
                                VertexData::new(self.lcs.u.a.x, self.lcs.u.a.y, color),
                                VertexData::new(self.lcs.l.a.x, self.lcs.l.a.y, color),
                                VertexData::new(self.lcs.l_aa.a.x, self.lcs.l_aa.a.y, transparent_color),

                                VertexData::new(upper_aa.x, upper_aa.y, transparent_color),
                                VertexData::new(upper.x, upper.y, color),
                                VertexData::new(self.lcs.l.b.x, self.lcs.l.b.y, color),
                                VertexData::new(self.lcs.l_aa.b.x, self.lcs.l_aa.b.y, transparent_color),
                                VertexData::new(self.rcs.l.a.x, self.rcs.l.a.y, color),

                                VertexData::new(self.rcs.l_aa.a.x, self.rcs.l_aa.a.y, transparent_color),
                                VertexData::new(self.rcs.u_aa.b.x, self.rcs.u_aa.b.y, transparent_color),
                                VertexData::new(self.rcs.u.b.x, self.rcs.u.b.y, color),
                                VertexData::new(self.rcs.l.b.x, self.rcs.l.b.y, color),
                                VertexData::new(self.rcs.l_aa.b.x, self.rcs.l_aa.b.y, transparent_color),
                            ],
                            indices: make_indices! {
                                * 0 4 1 5;
                                * 1 5 2 6;
                                * 2 6 3 7;

                                * 4 10 5 11;
                                * 5 11 8 12;
                                * 8 12 9 13;

                                * 6 8 7 9;
                                + 6 5 8
                            },
                        }
                    }
                    (true, JointStyle::Bevel) => {
                        SegmentTriangulation::Bevel {
                            vertices: [
                                VertexData::new(self.lcs.u_aa.a.x, self.lcs.u_aa.a.y, transparent_color),
                                VertexData::new(self.lcs.u.a.x, self.lcs.u.a.y, color, ),
                                VertexData::new(self.lcs.l.a.x, self.lcs.l.a.y, color, ),
                                VertexData::new(self.lcs.l_aa.a.x, self.lcs.l_aa.a.y, transparent_color),

                                VertexData::new(self.lcs.u_aa.b.x, self.lcs.u_aa.b.y, transparent_color),
                                VertexData::new(self.lcs.u.b.x, self.lcs.u.b.y, color),
                                VertexData::new(self.rcs.u_aa.a.x, self.rcs.u_aa.a.y, transparent_color),
                                VertexData::new(self.rcs.u.a.x, self.rcs.u.a.y, color),
                                VertexData::new(lower.x, lower.y, color),
                                VertexData::new(lower_aa.x, lower_aa.y, transparent_color),

                                VertexData::new(self.rcs.u_aa.b.x, self.rcs.u_aa.b.y, transparent_color),
                                VertexData::new(self.rcs.u.b.x, self.rcs.u.b.y, color),
                                VertexData::new(self.rcs.l.b.x, self.rcs.l.b.y, color),
                                VertexData::new(self.rcs.l_aa.b.x, self.rcs.l_aa.b.y, transparent_color)
                            ],
                            indices: make_indices! {
                                * 0 4 1 5;
                                * 1 5 2 8;
                                * 2 8 3 9;

                                * 6 10 7 11;
                                * 7 11 8 12;
                                * 8 12 9 13;

                                * 4 6 5 7;
                                + 8 5 7
                            },
                        }
                    }
                    (_, JointStyle::Miter) => {
                        SegmentTriangulation::Miter {
                            vertices: [
                                VertexData::new(self.lcs.u_aa.a.x, self.lcs.u_aa.a.y, transparent_color),
                                VertexData::new(self.lcs.u.a.x, self.lcs.u.a.y, color),
                                VertexData::new(self.lcs.l.a.x, self.lcs.l.a.y, color),
                                VertexData::new(self.lcs.l_aa.a.x, self.lcs.l_aa.a.y, transparent_color),

                                VertexData::new(upper_aa.x, upper_aa.y, transparent_color),
                                VertexData::new(upper.x, upper.y, color),
                                VertexData::new(lower.x, lower.y, color),
                                VertexData::new(lower_aa.x, lower_aa.y, transparent_color),

                                VertexData::new(self.rcs.u_aa.b.x, self.rcs.u_aa.b.y, transparent_color),
                                VertexData::new(self.rcs.u.b.x, self.rcs.u.b.y, color),
                                VertexData::new(self.rcs.l.b.x, self.rcs.l.b.y, color),
                                VertexData::new(self.rcs.l_aa.b.x, self.rcs.l_aa.b.y, transparent_color)
                            ],
                            indices: make_indices! {
                                * 0 4 1 5;
                                * 1 5 2 6;
                                * 2 6 3 7;

                                * 4 8 5 9;
                                * 5 9 6 10;
                                * 6 10 7 11
                            },
                        }
                    }
                }
            }
        }
    }
}

impl CapSegment {
    pub(crate) fn new(a: Point2<f32>, b: Point2<f32>, thickness: f32) -> Self {
        let line_segment = LineSegment { a, b };
        Self(PolySegment::new(&line_segment, thickness), thickness / 2.0)
    }

    pub(crate) fn triangulate(&self, start_id: u16, color: Color, style: EndCapStyle) -> SegmentTriangulation {
        let transparent_color = (color.r, color.g, color.b, color.a);
        let color = (color.r, color.g, color.b, color.a);
        let dir_norm = self.0.c.direction(Direction::Normalized);
        match style {
            EndCapStyle::Butt => SegmentTriangulation::Straight {
                vertices: [
                    VertexData::new(self.0.u_aa.a.x, self.0.u_aa.a.y, transparent_color),
                    VertexData::new(self.0.u.a.x, self.0.u.a.y, color),
                    VertexData::new(self.0.l.a.x, self.0.l.a.y, color),
                    VertexData::new(self.0.l_aa.a.x, self.0.l_aa.a.y, transparent_color),

                    VertexData::new(
                        self.0.u_aa.b.x + dir_norm.x,
                        self.0.u_aa.b.y + dir_norm.y,
                        transparent_color
                    ),
                    VertexData::new(self.0.u.b.x, self.0.u.b.y, color),
                    VertexData::new(self.0.l.b.x, self.0.l.b.y, color),
                    VertexData::new(
                        self.0.l_aa.b.x + dir_norm.x,
                        self.0.l_aa.b.y + dir_norm.y,
                        transparent_color,
                    ),
                ],
                indices: make_indices! {
                    * 0 4 1 5;
                    * 1 5 2 6;
                    * 2 6 3 7;
                    * 0 1 2 3;
                    * 4 5 6 7;
                },
            },
            EndCapStyle::Square => SegmentTriangulation::Straight {
                vertices: [
                    VertexData::new(self.0.u_aa.a.x, self.0.u_aa.a.y, transparent_color),
                    VertexData::new(self.0.u.a.x, self.0.u.a.y, color),
                    VertexData::new(self.0.l.a.x, self.0.l.a.y, color),
                    VertexData::new(self.0.l_aa.a.x, self.0.l_aa.a.y, transparent_color),

                    VertexData::new(
                        self.0.u_aa.b.x + dir_norm.x * (self.1 + 1.0),
                        self.0.u_aa.b.y + dir_norm.y * (self.1 + 1.0),
                        transparent_color
                    ),
                    VertexData::new(
                        self.0.u.b.x + dir_norm.x * self.1,
                        self.0.u.b.y + dir_norm.y * self.1,
                        color
                    ),
                    VertexData::new(
                        self.0.l.b.x + dir_norm.x * self.1,
                        self.0.l.b.y + dir_norm.y * self.1,
                        color
                    ),
                    VertexData::new(
                        self.0.l_aa.b.x + dir_norm.x * (self.1 + 1.0),
                        self.0.l_aa.b.y + dir_norm.y * (self.1 + 1.0),
                        transparent_color
                    ),
                ],
                indices: make_indices! {
                    * 0 4 1 5;
                    * 1 5 2 6;
                    * 2 6 3 7;
                    * 0 1 2 3;
                    * 4 5 6 7;
                },
            }
        }
    }
}

impl DoubleCapSegment {
    pub(crate) fn new(a: Point2<f32>, b: Point2<f32>, thickness: f32) -> Self {
        let line_segment = LineSegment { a, b };
        Self(PolySegment::new(&line_segment, thickness), thickness / 2.0)
    }

    pub(crate) fn triangulate(&self, start_id: u16, color: Color, style: EndCapStyle) -> SegmentTriangulation {
        let transparent_color = (color.r, color.g, color.b, color.a);
        let color = (color.r, color.g, color.b, color.a);
        let dir_norm = self.0.c.direction(Direction::Normalized);
        match style {
            EndCapStyle::Butt => SegmentTriangulation::Straight {
                vertices: [
                    VertexData::new(
                        self.0.u_aa.a.x - dir_norm.x,
                        self.0.u_aa.a.y - dir_norm.y,
                        transparent_color,
                    ),
                    VertexData::new(self.0.u.a.x, self.0.u.a.y, color),
                    VertexData::new(self.0.l.a.x, self.0.l.a.y, color),
                    VertexData::new(
                        self.0.l_aa.a.x - dir_norm.x,
                        self.0.l_aa.a.y - dir_norm.y,
                        transparent_color,
                    ),
                    VertexData::new(
                        self.0.u_aa.b.x + dir_norm.x,
                        self.0.u_aa.b.y + dir_norm.y,
                        transparent_color,
                    ),
                    VertexData::new(self.0.u.b.x, self.0.u.b.y, color),
                    VertexData::new(self.0.l.b.x, self.0.l.b.y, color),
                    VertexData::new(
                        self.0.l_aa.b.x + dir_norm.x,
                        self.0.l_aa.b.y + dir_norm.y,
                        transparent_color,
                    )
                ],
                indices: make_indices! {
                    * 0 4 1 5;
                    * 1 5 2 6;
                    * 2 6 3 7;
                    * 0 1 2 3;
                    * 4 5 6 7;
                },
            },
            EndCapStyle::Square => SegmentTriangulation::Straight {
                vertices: [
                    VertexData::new(
                        self.0.u_aa.a.x - dir_norm.x * (self.1 + 1.0),
                        self.0.u_aa.a.y - dir_norm.y * (self.1 + 1.0),
                        transparent_color,
                    ),
                    VertexData::new(
                        self.0.u.a.x - dir_norm.x * self.1,
                        self.0.u.a.y - dir_norm.y * self.1,
                        color,
                    ),
                    VertexData::new(
                        self.0.l.a.x - dir_norm.x * self.1,
                        self.0.l.a.y - dir_norm.y * self.1,
                        color,
                    ),
                    VertexData::new(
                        self.0.l_aa.a.x - dir_norm.x * (self.1 + 1.0),
                        self.0.l_aa.a.y - dir_norm.y * (self.1 + 1.0),
                        transparent_color,
                    ),
                    VertexData::new(
                        self.0.u_aa.b.x + dir_norm.x * (self.1 + 1.0),
                        self.0.u_aa.b.y + dir_norm.y * (self.1 + 1.0),
                        transparent_color,
                    ),
                    VertexData::new(
                        self.0.u.b.x + dir_norm.x * self.1,
                        self.0.u.b.y + dir_norm.y * self.1,
                        color,
                    ),
                    VertexData::new(
                        self.0.l.b.x + dir_norm.x * self.1,
                        self.0.l.b.y + dir_norm.y * self.1,
                        color,
                    ),
                    VertexData::new(
                        self.0.l_aa.b.x + dir_norm.x * (self.1 + 1.0),
                        self.0.l_aa.b.y + dir_norm.y * (self.1 + 1.0),
                        transparent_color,
                    ),
                ],
                indices: make_indices! {
                    * 0 4 1 5;
                    * 1 5 2 6;
                    * 2 6 3 7;
                    * 0 1 2 3;
                    * 4 5 6 7;
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::poly_line_2d::parts::VSegments;

    #[test]
    pub fn test_clockwise_checker() {
        let v_segments = VSegments::new(
            [0.0, 0.0].into(),
            [0.0, 100.0].into(),
            [100.0, 85.0].into(),
            8.0,
        );
        assert_eq!(true, v_segments.is_clockwise());

        let v_segments = VSegments::new(
            [0.0, 0.0].into(),
            [0.0, 100.0].into(),
            [-100.0, 85.0].into(),
            8.0,
        );
        assert_eq!(false, v_segments.is_clockwise());
    }
}