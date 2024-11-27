fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let builder = bindgen_cuda::Builder::default().include_paths_glob("../cpp_header/*");
    println!("cargo:info={builder:?}");
    let bindings = builder.build_ptx().unwrap();
    bindings.write("src/lib.rs").unwrap();
}
