use std::path::Path;

use bespoke_engine::resource_loader::generate_resources;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    generate_resources(Path::new("src/res"));
}//edit