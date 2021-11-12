extern "C" {
    pub fn cpp_main();
}

fn main() {
    unsafe {
        cpp_main();
    }
}
