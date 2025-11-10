use crate::ecs::entities::Entity;

pub struct SparseSet<T> {
    sparse: Vec<usize>,
    dense: Vec<T>,
    entities: Vec<Entity>,
}

impl<T> SparseSet<T> {
    const TOMBSTONE: usize = u16::MAX as usize;

    pub fn new(capacity: usize) -> Self {
        Self {
            sparse: vec![Self::TOMBSTONE; capacity],
            dense: Vec::new(),
            entities: Vec::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, component: T) {
        let index = entity.index as usize;
        if index >= Self::TOMBSTONE {
            return;
        }

        self.sparse[index] = self.dense.len();
        self.dense.push(component);
        self.entities.push(entity);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Entity, &T)> {
        self.entities.iter().zip(self.dense.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Entity, &mut T)> {
        self.entities.iter().zip(self.dense.iter_mut())
    }

    pub fn get(&self, entity: &Entity) -> Option<&T> {
        let index = *self.sparse.get(entity.index as usize)?;

        if index == Self::TOMBSTONE {
            return None;
        }

        if self.entities[index].generation != entity.generation {
            return None;
        }

        Some(&self.dense[index])
    }

    pub fn get_mut(&mut self, entity: &Entity) -> Option<&mut T> {
        let index = *self.sparse.get(entity.index as usize)?;

        if index == Self::TOMBSTONE {
            return None;
        }

        if self.entities[index].generation != entity.generation {
            return None;
        }

        Some(&mut self.dense[index])
    }
}
