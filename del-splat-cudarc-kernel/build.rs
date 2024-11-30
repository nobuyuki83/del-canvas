fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let path_out_dir = std::path::Path::new(&out_dir).join("cpp_headers");
    // dbg!(&path_out_dir);
    std::fs::create_dir_all(&path_out_dir).unwrap();
    del_geo_cpp_headers::HEADERS.write_files(&path_out_dir);
    let glob_input = path_out_dir.join("*.h").into_os_string().into_string().unwrap();
    // dbg!(&glob_input);
    let builder = bindgen_cuda::Builder::default().include_paths_glob(&glob_input);
    println!("cargo:info={builder:?}");
    let bindings = builder.build_ptx().unwrap();
    bindings.write("src/lib.rs").unwrap();
}
