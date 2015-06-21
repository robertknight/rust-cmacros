use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path};
use walker::Walker;

extern crate argparse;
extern crate walker;
extern crate cmacros;

#[macro_use]
mod util;

fn main() {
    let mut input_dir = String::new();
    {
        let mut parser = argparse::ArgumentParser::new();
        parser.set_description("Parse all headers in a given directory and output all macro definitions found");
        parser
          .refer(&mut input_dir)
          .add_argument("dir", argparse::Store, "Directory containing headers to parse")
          .required();
        parser.parse_args_or_exit();
    }
    let input_root = Path::new(&input_dir);

    let walker = match Walker::new(&input_root) {
        Ok(walker) => walker,
        Err(err) => {
            fatal_err!("Unable to walk dir '{}': {}", input_root.to_string_lossy(), err);
        }
    };

    for entry in walker {
        if entry.is_err() {
            println_stderr!("Skipping {:?}", entry.err());
            continue;
        }

        let entry = entry.unwrap();
        match entry.path().extension() {
            Some(ext) if ext == "h" || ext == "hpp" => (),
            _ => {
                continue;
            }
        }

        let file_path = input_root.join(entry.path());
        let mut hdr = File::open(&file_path).unwrap();
        let mut src = String::new();
        match hdr.read_to_string(&mut src) {
            Ok(_) => {},
            Err(err) => {
                println_stderr!("Failed to parse {:?}: {}", file_path, err);
                continue;
            }
        }

        match cmacros::extract_macros(&src) {
            Ok(macros) => {
                for cmacro in macros {
                    let mut def = format!("#define {}", cmacro.name);
                    match cmacro.args {
                        Some(args) => def.push_str(&format!("({})", &args.connect(","))),
                        None => ()
                    }
                    match cmacro.body {
                        Some(body) => def.push_str(&format!(" {}", &body)),
                        None => ()
                    }
                    println!("{}", def);
                }
            },
            Err(err) => println_stderr!("Failed to parse {:?}: {}", file_path, err)
        }
    }
}

