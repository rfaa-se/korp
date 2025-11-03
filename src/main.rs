use korp::Korp;
use korp_engine::Engine;

fn main() {
    println!("Hello, korp!");
    Engine::new(12, Korp::new(), "korp").run();
}
