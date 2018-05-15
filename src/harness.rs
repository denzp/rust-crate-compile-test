#[macro_export]
macro_rules! crate_compile_test_suite {
    ($($name:expr => $block:block),*) => {
        #[allow(non_upper_case_globals)]
        fn main() {
            pub static mut is_success: bool = true;

            use std::process::exit;
            use std::io::stdout;

            $(
                println!("Running `{}`", $name);
                $block;
                println!("");
            )*

            if unsafe { !is_success } {
                exit(1);
            }
        }
    };
}

#[macro_export]
macro_rules! run_compile_tests {
    ($config:expr) => {
        if run_tests_with_writer($config, stdout()).is_err() {
            unsafe {
                is_success = false;
            }
        }
    };
}
