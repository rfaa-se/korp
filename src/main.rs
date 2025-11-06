use korp::Korp;
use korp_engine::Engine;

fn main() {
    println!("Hello, korp!");

    let degs = [-50, 0, 50, 100, 150, 200, 250, 300, 350, 400];
    for deg in degs {
        let d = korp_math::Vec2::new(
            korp_math::Flint::from_i16(0),
            korp_math::Flint::from_i16(-1),
        );
        let dd = korp_math::Vec2::new(0.0, -1.0);

        let r = korp_math::Flint::from_i16(deg).to_radians();
        let rr = (deg as f32).to_radians();

        let (s, c) = r.sin_cos();
        let (ss, cc) = rr.sin_cos();

        println!(
            "I D {:3} | DIR {},{} | RAD {:+.20} | SINCOS {:+.20}, {:+.20}",
            deg,
            d.x.to_f32(),
            d.y.to_f32(),
            r.to_f32(),
            s.to_f32(),
            c.to_f32()
        );
        println!(
            "F D {:3} | DIR {},{} | RAD {:+.20} | SINCOS {:+.20}, {:+.20}",
            deg, dd.x, dd.y, rr, ss, cc
        );
        println!("-------------------------------------------------------------------------------");
    }

    Engine::new(12, Korp::new(), "korp").run();
}
