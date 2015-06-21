// http://stackoverflow.com/questions/27588416/how-to-send-output-to-stderr
macro_rules! println_stderr(
    ($($arg:tt)*) => ({
        use std::io::Write;
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    })
);

macro_rules! fatal_err(
    ($($arg:tt)*) => ({
        use std::process;
        println_stderr!($($arg)*);
        process::exit(1);
    })
);


