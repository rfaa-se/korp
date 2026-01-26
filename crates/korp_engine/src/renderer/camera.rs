use korp_math::Vec2;

pub struct Camera {
    half_width: f32,
    half_height: f32,
    position: Vec2<f32>,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            half_width: width * 0.5,
            half_height: height * 0.5,
            position: Vec2::new(0.0, 0.0),
        }
    }

    pub fn reposition(&mut self, position: Vec2<f32>) {
        self.position = position;
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.half_width = width * 0.5;
        self.half_height = height * 0.5;
    }

    pub(crate) fn view_projection(&self) -> [[f32; 4]; 4] {
        let left = self.position.x - self.half_width;
        let right = self.position.x + self.half_width;
        let top = self.position.y + self.half_height;
        let bottom = self.position.y - self.half_height;
        let near = 0.0;
        let far = 1.0;

        ortho(left, right, bottom, top, near, far)
    }
}

fn ortho(left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    [
        [2.0 / (right - left), 0.0, 0.0, 0.0],
        [0.0, 2.0 / (top - bottom), 0.0, 0.0],
        [0.0, 0.0, 1.0 / (far - near), 0.0],
        [
            -(right + left) / (right - left),
            -(top + bottom) / (top - bottom),
            -(near / (far - near)),
            1.0,
        ],
    ]
}
