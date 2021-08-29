use v8find4rs::v8_finder::{V8Finder, SearchPriority};

fn main() {
    let finder = V8Finder::new();
    let platform = finder.get_platform("8.3.8.2442", SearchPriority::X32);
    println!("{:?}", platform);
}