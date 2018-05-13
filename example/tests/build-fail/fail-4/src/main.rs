//~ GLOBAL-NOTE-REGEX -l(.+)libcore-(.+)\.rlib
//~ GLOBAL-NOTE-REGEX undefined reference to `some_external_fn'
//~ GLOBAL-NOTE-REGEX undefined reference to `third_external_fn'

extern "C" {
    fn some_external_fn();
    fn other_external_fn();
}

fn main() {
    unsafe {
        some_external_fn();
        other_external_fn();
    }
}
