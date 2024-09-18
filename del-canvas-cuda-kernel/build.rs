fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/xyzrgb.cu");
    println!("cargo:rerun-if-changed=src/pix2tri.cu");
    println!("cargo:rerun-if-changed=src/cumsum.cu");

    let builder = bindgen_cuda::Builder::default();
    println!("cargo:info={builder:?}");
    let bindings = builder.build_ptx().unwrap();
    bindings.write("src/lib.rs").unwrap();
}