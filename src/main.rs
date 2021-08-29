use v8find4rs::v8_finder::{V8Finder, SearchPriority};
use v8find4rs::v8_app::V8AppType;

fn main() {
    let finder = V8Finder::new();
    let platform = finder.get_platform("8.3.19", SearchPriority::X64);
    println!("{:?}", platform.unwrap().get_app_by_type(V8AppType::ThickClient));
}