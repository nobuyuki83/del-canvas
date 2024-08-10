
extern crate cc;

fn main() {
    cc::Build::new()
        .cuda(true)
        .flag("-cudart=shared")
        .flag("-gencode")
        .flag("arch=compute_86,code=sm_86")
        .files(["src/kernel.cu"])
        .compile("del_canvas_cuda");

    /* Link CUDA Runtime (libcudart.so) */

    // Add link directory
    // - This path depends on where you install CUDA (i.e. depends on your Linux distribution)
    // - This should be set by `$LIBRARY_PATH`
    // println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");
    // println!("cargo:rustc-link-search={}", std::path::Path::new("C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA\\v12.3\\lib\\x64").display());
    // println!("cargo:rustc-link-lib=cudart");

    /* Optional: Link CUDA Driver API (libcuda.so) */

    // println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64/stub");
    // println!("cargo:rustc-link-lib=cuda");
}
