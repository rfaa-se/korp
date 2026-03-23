use korp_engine::{color::Color, renderer::Renderer, shapes::Rectangle};
use korp_math::{Vec2, lerp};

use crate::ecs::components::{Components, traits::Renderable};

pub fn body(components: &Components, renderer: &mut Renderer, draw_filled: bool, alpha: f32) {
    for (_, body) in components.render.bodies.iter() {
        body.render(renderer, draw_filled, alpha);
    }
}

pub fn hitbox(components: &Components, renderer: &mut Renderer, alpha: f32) {
    for (_, hitbox) in components.render.hitboxes.iter() {
        let width = lerp(hitbox.old.width, hitbox.new.width, alpha);
        let height = lerp(hitbox.old.height, hitbox.new.height, alpha);
        let centroid = Vec2::new(
            lerp(hitbox.old.x, hitbox.new.x, alpha) + width * 0.5,
            lerp(hitbox.old.y, hitbox.new.y, alpha) + height * 0.5,
        );

        let rectangle = Rectangle::from(width, height, centroid);

        renderer.draw_rectangle_lines(rectangle, Vec2::new(1.0, 0.0), centroid, Color::BLUE);
    }
}

pub fn cosmos_bounds(components: &Components, renderer: &mut Renderer) {
    let bounds = components.render.cosmos;
    let rotation = Vec2::new(1.0, 0.0);
    let color = Color::RED;
    let origin = Vec2::new(
        bounds.x + bounds.width * 0.5,
        bounds.y + bounds.height * 0.5,
    );

    renderer.draw_rectangle_lines(bounds, rotation, origin, color);
}

pub fn quadtree_nodes(components: &Components, renderer: &mut Renderer) {
    for node in components.render.quadtree.iter() {
        let rotation = Vec2::new(1.0, 0.0);
        let color = Color::RED;
        let origin = Vec2::new(node.x + node.width * 0.5, node.y + node.height * 0.5);

        renderer.draw_rectangle_lines(*node, rotation, origin, color);
    }
}
