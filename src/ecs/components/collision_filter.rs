use crate::ecs::components::CollisionFilter;

impl CollisionFilter {
    pub const TRIANGLE: u32 = 1 << 0;
    pub const RECTANGLE: u32 = 1 << 1;
    pub const PROJECTILE: u32 = 1 << 2;

    pub fn is_collidable(&self, other: &CollisionFilter) -> bool {
        (self.mask & other.category) != 0 && (other.mask & self.category) != 0
    }
}
