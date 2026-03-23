use korp_engine::shapes::Rectangle;
use korp_math::Flint;

use crate::ecs::entities::Entity;

pub struct Quadtree {
    nodes: Vec<Node>,
    capacity: usize,
    bounds: Rectangle<Flint>,
    depth: usize,
}

pub struct Node {
    bounds: Rectangle<Flint>,
    kind: NodeKind,
    depth: usize,
}

enum NodeKind {
    Leaf {
        content: Vec<(Entity, Rectangle<Flint>)>,
    },
    Branch {
        indexes: [usize; 4],
    },
}

impl Quadtree {
    pub fn new(bounds: Rectangle<Flint>, capacity: usize, depth: usize) -> Self {
        Self {
            nodes: vec![Node {
                kind: NodeKind::Leaf {
                    content: Vec::new(),
                },
                bounds,
                depth: 0,
            }],
            capacity,
            bounds,
            depth,
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.nodes.push(Node {
            kind: NodeKind::Leaf {
                content: Vec::new(),
            },
            bounds: self.bounds,
            depth: 0,
        });
    }

    pub fn insert(&mut self, entity: Entity, hitbox: Rectangle<Flint>) {
        self.insert_into(0, entity, hitbox);
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    fn insert_into(&mut self, idx: usize, entity: Entity, hitbox: Rectangle<Flint>) {
        let node = &mut self.nodes[idx];

        if !node.bounds.overlaps(&hitbox) {
            return;
        }

        match node.kind {
            NodeKind::Leaf {
                content: ref mut entities_hitboxes,
            } => {
                if entities_hitboxes.len() < self.capacity || node.depth == self.depth {
                    entities_hitboxes.push((entity, hitbox));
                } else {
                    self.subdivide(idx);
                }
            }
            NodeKind::Branch { indexes } => {
                for index in indexes {
                    self.insert_into(index, entity, hitbox);
                }
            }
        }
    }

    fn subdivide(&mut self, idx: usize) {
        let offset = self.nodes.len();
        let kind = std::mem::replace(
            &mut self.nodes[idx].kind,
            NodeKind::Branch {
                indexes: core::array::from_fn(|i| offset + i),
            },
        );

        let NodeKind::Leaf { content } = kind else {
            panic!("wtf node kind");
        };

        let bounds = self.nodes[idx].bounds;
        let quads = quads(bounds);

        for (i, quad) in quads.iter().enumerate() {
            self.nodes.push(Node {
                bounds: *quad,
                kind: NodeKind::Leaf {
                    content: Vec::new(),
                },
                depth: self.nodes[idx].depth + 1,
            });

            for (entity, hitbox) in content.iter() {
                self.insert_into(offset + i, *entity, *hitbox);
            }
        }
    }
}

impl Node {
    pub fn content(&self) -> &[(Entity, Rectangle<Flint>)] {
        match self.kind {
            NodeKind::Leaf {
                content: ref entities_hitboxes,
            } => &entities_hitboxes,
            NodeKind::Branch { .. } => &[],
        }
    }

    pub fn bounds(&self) -> Rectangle<Flint> {
        self.bounds
    }
}

fn quads(bounds: Rectangle<Flint>) -> [Rectangle<Flint>; 4] {
    let hw = bounds.width * Flint::ZERO_FIVE;
    let hh = bounds.height * Flint::ZERO_FIVE;
    [
        Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: hw,
            height: hh,
        },
        Rectangle {
            x: bounds.x + hw,
            y: bounds.y,
            width: hw,
            height: hh,
        },
        Rectangle {
            x: bounds.x,
            y: bounds.y + hh,
            width: hw,
            height: hh,
        },
        Rectangle {
            x: bounds.x + hw,
            y: bounds.y + hh,
            width: hw,
            height: hh,
        },
    ]
}
