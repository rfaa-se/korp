#[derive(Copy, Clone)]
pub struct Morph<T> {
    pub old: T,
    pub new: T,
}

impl<T> Morph<T> {
    pub const fn new(old: T, new: T) -> Self {
        Self { old, new }
    }

    pub const fn one(value: T) -> Self
    where
        T: Copy + Clone,
    {
        Self {
            old: value,
            new: value,
        }
    }
}
