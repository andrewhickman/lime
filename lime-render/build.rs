fn main() {
    println!("cargo:rerun-if-changed=shaders/d2/frag.glsl");
    println!("cargo:rerun-if-changed=shaders/d2/vert.glsl");
}
