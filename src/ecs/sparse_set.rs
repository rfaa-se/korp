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
        let sparse_index = entity.index as usize;
        if sparse_index >= Self::TOMBSTONE {
            return;
        }

        // replace component if entity already has it
        if let Some(c) = self.get_mut(&entity) {
            *c = component;
            return;
        }

        self.sparse[sparse_index] = self.dense.len();
        self.dense.push(component);
        self.entities.push(entity);
    }

    pub fn remove(&mut self, entity: Entity) {
        let sparse_index = entity.index as usize;
        if sparse_index >= Self::TOMBSTONE {
            return;
        }

        let dense_index = self.sparse[sparse_index];
        if dense_index == Self::TOMBSTONE {
            return;
        }

        if self.entities[dense_index].generation != entity.generation {
            return;
        }

        self.sparse[sparse_index] = Self::TOMBSTONE;
        let last_index = self.dense.len() - 1;

        if dense_index != last_index {
            self.dense.swap(dense_index, last_index);
            self.entities.swap(dense_index, last_index);

            let moved = self.entities[dense_index];
            self.sparse[moved.index as usize] = dense_index;
        }

        self.dense.pop();
        self.entities.pop();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Entity, &T)> {
        self.entities.iter().zip(self.dense.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Entity, &mut T)> {
        self.entities.iter().zip(self.dense.iter_mut())
    }

    pub fn len(&self) -> usize {
        self.dense.len()
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
