use std::fs::File;
use std::io::{Read};

extern crate argparse;
extern crate cmacros;

#[macro_use]
mod util;

fn main() {
    let mut input_path = String::new();

    {
        let mut parser = argparse::ArgumentParser::new();
        parser.set_description("Reads a header file and outputs a translation of simple (non-function) macros to Rust constants");
        parser
          .refer(&mut input_path)
          .add_argument("input", argparse::Store, "Input header file to parse")
          .required();
        parser.parse_args_or_exit();
    }

    let mut header_file = File::open(&input_path).unwrap();
    let mut header_src = String::new();
    match header_file.read_to_string(&mut header_src) {
        Err(e) => panic!("Failed to read header file: {}", e),
        _ => ()
    }
    let macros = match cmacros::extract_macros(&header_src) {
        Ok(macros) => macros,
        Err(err) => fatal_err!("Failed to extract macros from {}: {}", &input_path, err)
    };

    let rust_src = cmacros::generate_rust_src(&macros, |def| cmacros::translate_macro(def));
    println!("{}", rust_src);
}

