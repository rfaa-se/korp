#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Entity {
    pub index: u32,
    pub generation: u32,
}

pub struct EntityFactory {
    generations: Vec<u32>,
    free: Vec<u32>,
}

impl EntityFactory {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn create(&mut self) -> Entity {
        if let Some(index) = self.free.pop() {
            let generation = self.generations[index as usize];

            Entity { index, generation }
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(0);

            Entity {
                index,
                generation: 0,
            }
        }
    }

    pub fn destroy(&mut self, entity: Entity) {
        if let Some(generation) = self.generations.get_mut(entity.index as usize) {
            *generation += 1;
            self.free.push(entity.index);
        }
    }
}
