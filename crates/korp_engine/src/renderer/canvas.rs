use korp_math::Vec2;

use crate::{
    color::Color,
    misc::Morph,
    renderer::{Camera, Vertex},
    shapes::{Line, Rectangle, Triangle},
};

pub struct Canvas {
    pub(super) vertices: Vec<Vertex>,
    pub(super) clear_color: wgpu::Color,
}

pub struct ScopeCanvas<'a> {
    pub canvas: &'a mut Canvas,
    pub camera: &'a Camera,
}

impl Drop for ScopeCanvas<'_> {
    fn drop(&mut self) {
        // self.canvas.
    }
}

impl Canvas {
    pub(crate) fn new() -> Self {
        Self {
            vertices: Vec::new(),
            clear_color: wgpu::Color::BLACK,
        }
    }

    pub(crate) fn prepare(&mut self) {
        self.vertices.clear();
    }

    pub fn begin<'a>(&'a mut self, camera: &'a Camera) -> ScopeCanvas<'a> {
        ScopeCanvas {
            canvas: self,
            camera,
        }
    }

    pub fn draw_line(
        &mut self,
        line: Morph<Line<f32>>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        let t = |a: Vec2<f32>, b: Vec2<f32>| {
            let dir = b - a;
            let norm = dir.perp().normalized() * 0.5;
            (a + norm, b + norm, b - norm, a - norm)
        };

        let (ov0, ov1, ov2, ov3) = t(line.old.start, line.old.end);
        let (nv0, nv1, nv2, nv3) = t(line.new.start, line.new.end);

        let v0 = Vertex {
            position_old: ov0.into(),
            position_new: nv0.into(),
            rotation_old: rotation.old.into(),
            rotation_new: rotation.new.into(),
            origin_old: origin.old.into(),
            origin_new: origin.new.into(),
            color_old: color.old.into(),
            color_new: color.new.into(),
        };

        let v1 = Vertex {
            position_old: ov1.into(),
            position_new: nv1.into(),
            ..v0
        };

        let v2 = Vertex {
            position_old: ov2.into(),
            position_new: nv2.into(),
            ..v0
        };

        let v3 = Vertex {
            position_old: ov3.into(),
            position_new: nv3.into(),
            ..v0
        };

        self.vertices.push(v0);
        self.vertices.push(v1);
        self.vertices.push(v2);
        self.vertices.push(v2);
        self.vertices.push(v3);
        self.vertices.push(v0);
    }

    pub fn draw_triangle_filled(
        &mut self,
        triangle: Morph<Triangle<f32>>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        let top = Vertex {
            position_old: triangle.old.top.into(),
            position_new: triangle.new.top.into(),
            rotation_old: rotation.old.into(),
            rotation_new: rotation.new.into(),
            origin_old: origin.old.into(),
            origin_new: origin.new.into(),
            color_old: color.old.into(),
            color_new: color.new.into(),
        };

        let left = Vertex {
            position_old: triangle.old.left.into(),
            position_new: triangle.new.left.into(),
            ..top
        };

        let right = Vertex {
            position_old: triangle.old.right.into(),
            position_new: triangle.new.right.into(),
            ..top
        };

        self.vertices.push(top);
        self.vertices.push(left);
        self.vertices.push(right);
    }

    pub fn draw_triangle_lines(
        &mut self,
        triangle: Morph<Triangle<f32>>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        let vertices = [
            (triangle.old.top, triangle.new.top),
            (triangle.old.left, triangle.new.left),
            (triangle.old.right, triangle.new.right),
        ];

        for i in 0..vertices.len() {
            // TODO: the lines don't match up 100% with the filled version, fixable?
            // let l = Line {
            //     start: vertices[i].0
            //         + match i {
            //             0 => Vec2::new(0.5, 0.),
            //             1 => Vec2::new(0.5, -0.5), // OK
            //             _ => Vec2::new(-0.5, 0.),
            //         },
            //     end: vertices[(i + 1) % 3].0
            //         + match i {
            //             0 => Vec2::new(0.5, -0.),
            //             1 => Vec2::new(-0.5, -0.5), // OK
            //             _ => Vec2::new(-0.5, 0.),
            //         },
            // };
            self.draw_line(
                Morph {
                    old: Line {
                        start: vertices[i].0,
                        end: vertices[(i + 1) % 3].0,
                    },
                    new: Line {
                        start: vertices[i].1,
                        end: vertices[(i + 1) % 3].1,
                    },
                },
                rotation,
                origin,
                color,
            );
        }
    }

    pub fn draw_rectangle_filled(
        &mut self,
        rect: Morph<Rectangle<f32>>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        let v1 = Vertex {
            position_old: [rect.old.x, rect.old.y],
            position_new: [rect.new.x, rect.new.y],
            rotation_old: rotation.old.into(),
            rotation_new: rotation.new.into(),
            origin_old: origin.old.into(),
            origin_new: origin.new.into(),
            color_old: color.old.into(),
            color_new: color.new.into(),
        };

        let v2 = Vertex {
            position_old: [rect.old.x + rect.old.width, rect.old.y],
            position_new: [rect.new.x + rect.new.width, rect.new.y],
            ..v1
        };

        let v3 = Vertex {
            position_old: [rect.old.x, rect.old.y + rect.old.height],
            position_new: [rect.new.x, rect.new.y + rect.new.height],
            ..v1
        };

        let v4 = Vertex {
            position_old: [rect.old.x + rect.old.width, rect.old.y + rect.old.height],
            position_new: [rect.new.x + rect.new.width, rect.new.y + rect.new.height],
            ..v1
        };

        self.vertices.push(v1);
        self.vertices.push(v2);
        self.vertices.push(v3);
        self.vertices.push(v2);
        self.vertices.push(v4);
        self.vertices.push(v3);
    }

    pub fn draw_rectangle_lines(
        &mut self,
        rect: Morph<Rectangle<f32>>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        self.draw_line(
            Morph {
                old: Line {
                    start: Vec2::new(rect.old.x, rect.old.y + 0.5),
                    end: Vec2::new(rect.old.x + rect.old.width, rect.old.y + 0.5),
                },
                new: Line {
                    start: Vec2::new(rect.new.x, rect.new.y + 0.5),
                    end: Vec2::new(rect.new.x + rect.new.width, rect.new.y + 0.5),
                },
            },
            rotation,
            origin,
            color,
        );

        self.draw_line(
            Morph {
                old: Line {
                    start: Vec2::new(rect.old.x + rect.old.width - 0.5, rect.old.y),
                    end: Vec2::new(
                        rect.old.x + rect.old.width - 0.5,
                        rect.old.y + rect.old.height,
                    ),
                },
                new: Line {
                    start: Vec2::new(rect.new.x + rect.new.width - 0.5, rect.new.y),
                    end: Vec2::new(
                        rect.new.x + rect.new.width - 0.5,
                        rect.new.y + rect.new.height,
                    ),
                },
            },
            rotation,
            origin,
            color,
        );

        self.draw_line(
            Morph {
                old: Line {
                    start: Vec2::new(
                        rect.old.x + rect.old.width,
                        rect.old.y + rect.old.height - 0.5,
                    ),
                    end: Vec2::new(rect.old.x, rect.old.y + rect.old.height - 0.5),
                },
                new: Line {
                    start: Vec2::new(
                        rect.new.x + rect.new.width,
                        rect.new.y + rect.new.height - 0.5,
                    ),
                    end: Vec2::new(rect.new.x, rect.new.y + rect.new.height - 0.5),
                },
            },
            rotation,
            origin,
            color,
        );

        self.draw_line(
            Morph {
                old: Line {
                    start: Vec2::new(rect.old.x + 0.5, rect.old.y + rect.old.height),
                    end: Vec2::new(rect.old.x + 0.5, rect.old.y),
                },
                new: Line {
                    start: Vec2::new(rect.new.x + 0.5, rect.new.y + rect.new.height),
                    end: Vec2::new(rect.new.x + 0.5, rect.new.y),
                },
            },
            rotation,
            origin,
            color,
        );
    }
}
