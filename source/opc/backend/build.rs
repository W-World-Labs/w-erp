use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../frontend/dist");
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let index = Path::new("../frontend/dist/index.html");
    if profile == "release" && !index.exists() {
        panic!(
            "../frontend/dist/index.html missing — run `npm run build` in the frontend/ directory before a release build (the SPA is embedded into the binary)."
        );
    }
    if !index.exists() {
        println!(
            "cargo:warning=frontend/dist/ not built; SPA routes will 404 until `npm run build` is run in frontend/."
        );
    }
}
