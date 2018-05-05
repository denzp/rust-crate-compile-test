macro_rules! read_output {
    ($path:expr) => {{
        use std::fs::File;
        use std::io::{BufReader, Read};

        let mut contents = String::new();
        let mut reader = BufReader::new(File::open($path).unwrap());

        reader.read_to_string(&mut contents).unwrap();
        contents
    }};
}

mod build;
mod check_errors;
