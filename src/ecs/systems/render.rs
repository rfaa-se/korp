use korp_engine::{color::Color, renderer::Renderer, shapes::Rectangle};
use korp_math::{Flint, Vec2, lerp};

use crate::ecs::components::{Components, traits::Renderable};

// pub fn body(components: &Components, renderer: &mut Renderer, toggle: bool, alpha: f32) {
//     for (_, body) in components.render.bodies.iter() {
//         body.render(renderer, toggle, alpha);
//     }
// }

// pub fn hitbox(components: &Components, renderer: &mut Renderer, alpha: f32) {
//     for (_, hitbox) in components.render.hitboxes.iter() {
//         let width = lerp(hitbox.old.width, hitbox.new.width, alpha);
//         let height = lerp(hitbox.old.height, hitbox.new.height, alpha);
//         let centroid = Vec2::new(
//             lerp(hitbox.old.x, hitbox.new.x, alpha) + width * 0.5,
//             lerp(hitbox.old.y, hitbox.new.y, alpha) + height * 0.5,
//         );

//         let rectangle = Rectangle::from(width, height, centroid);

//         renderer.draw_rectangle_lines(rectangle, Vec2::new(1.0, 0.0), centroid, Color::BLUE);
//     }
// }

// pub fn cosmos(dimensions: &Rectangle<Flint>, renderer: &mut Renderer) {
//     let dimensions = Rectangle {
//         x: dimensions.x.into(),
//         y: dimensions.y.into(),
//         width: dimensions.width.into(),
//         height: dimensions.height.into(),
//     };

//     let rotation = Vec2::new(1.0, 0.0);

//     let origin = Vec2::new(
//         dimensions.x + dimensions.width * 0.5,
//         dimensions.y + dimensions.height * 0.5,
//     );

//     let color = Color::RED;

//     renderer.draw_rectangle_lines(dimensions, rotation, origin, color);
// }
