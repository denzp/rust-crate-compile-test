macro_rules! div_n {
    ($n:expr) => {
        pub fn div_n(a: f64) -> f64 {
            a / $n as f64
        }
    };
}

//~ EXPAND pub fn div_n(a: f64) -> f64 {
//~ EXPAND     a / 4 as f64
//~ EXPAND }

div_n!(5);
