use std::fs;
use std::fs::File;
use std::io::{Read, Write};

extern crate cmacros;

fn main() {
    let input_path = "/usr/include/sqlite3.h";
    let mut header_file = File::open(input_path).unwrap();
    let mut header_src = String::new();
    match header_file.read_to_string(&mut header_src) {
        Err(e) => panic!("Failed to read header file: {}", e),
        _ => ()
    }
    let macros = cmacros::extract_macros(&header_src).unwrap();
    let skipped_macros = [
        "SQLITE_EXTERN",
        "SQLITE_STATIC",
        "SQLITE_TRANSIENT",
        "double"
    ];
    let src = cmacros::generate_rust_src(&macros, |def| {
        if skipped_macros.contains(&&def.name[..]) {
            cmacros::TranslateAction::Skip
        } else {
            cmacros::translate_macro(def)
        }
    });
    let output_path = "sqlite3.rs";
    let mut out_f = File::create(output_path).unwrap();
    out_f.write_all(src.as_bytes()).ok();
    println!("Generated {}", output_path);
}

