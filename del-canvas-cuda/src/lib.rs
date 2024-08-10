#[link(name = "del_canvas_cuda", kind = "static")]
extern "C" {
    fn vectorAdd_main();
}

pub fn hoge() {
    unsafe {
        vectorAdd_main();
    }
}

