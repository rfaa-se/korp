mod f32;
mod flint;

#[derive(Copy, Clone, Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> From<Vec2<T>> for [T; 2] {
    fn from(value: Vec2<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T> Vec2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}
