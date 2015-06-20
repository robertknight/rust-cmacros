use walker::Walker;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};

extern crate walker;
extern crate cmacros;

// http://stackoverflow.com/questions/27588416/how-to-send-output-to-stderr
macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

fn main() {
    let include_root = Path::new("/usr/include");
    for entry in Walker::new(include_root).unwrap() {
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

        let file_path = include_root.join(entry.path());
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

