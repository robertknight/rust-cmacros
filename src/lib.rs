//! Library providing functions to assist with parsing and translating
//! '#define' macro definitions from C header files to corresponding
//! Rust code for use with bindings to external libraries.

/// Macro definition parsed from a C header file
#[derive(PartialEq, Debug)]
pub struct CMacro {
    /// The name of the macro
    pub name: String,
    /// The arguments to the macro if it is a function-like macro
    pub args: Option<Vec<String>>,
    /// The text that the macro expands to
    pub body: Option<String>
}

/// Attributes for a Rust constant definition
pub struct ConstDecl {
    pub name: String,
    pub type_name: String,
    pub expr: String
}

/// Specifies a transformation
/// from a C macro definition to Rust code
pub enum TranslateAction {
    /// Generate a constant with a specified type
    TypedConst(ConstDecl),
    /// Do not generate anything for this macro
    Skip
}

/// Provides a view of a string as a stream
/// of chars which can be peeked, consumed etc.
/// for use when writing simple parsers.
struct CharStream<'a> {
    input: &'a str,
    pos: usize
}

impl<'a> CharStream<'a> {
    fn new(input: &str) -> CharStream {
        CharStream{input: input, pos: 0}
    }

    fn at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek(&self, offset: usize) -> char {
        self.tail().chars().nth(offset).unwrap_or(0 as char)
    }

    fn next(&mut self) -> char {
        let ch = self.peek(0);
        self.pos += 1;
        ch
    }

    fn consume(&mut self, text: &str) -> bool {
        if self.tail().starts_with(text) {
            self.pos += text.len();
            true
        } else {
            false
        }
    }

    fn consume_char(&mut self, required: char) -> bool {
        self.consume_while(|ch| ch == required).len() > 0
    }

    fn consume_while<Predicate>(&mut self, test: Predicate) -> &'a str 
    where Predicate: Fn(char) -> bool {
        let start_pos = self.pos;
        while test(self.peek(0)) {
            self.next();
        }
        &self.input[start_pos..self.pos]
    }

    fn skip_whitespace(&mut self) -> &str {
        self.consume_while(|ch| ch.is_whitespace())
    }

    fn tail(&self) -> &str {
        &self.input[self.pos..]
    }
}

/// Iterator over lines in a C header file.
/// Lines with a trailing '\' are concatenated into
/// single lines
struct CHeaderLineIter<'a> {
    input: CharStream<'a>
}

impl<'a> Iterator for CHeaderLineIter<'a> {
    type Item = String;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.input.at_end() {
            None
        } else {
            self.input.skip_whitespace();

            let mut line = String::new();
            while !self.input.at_end() {
                let ch = self.input.peek(0);
                if ch == '\n' {
                    self.input.next();
                    break;
                } else if ch == '\\' && self.input.peek(1) == '\n' {
                    self.input.next();
                    self.input.next();
                } else {
                    self.input.next();
                    line.push(ch);
                }
            }
            Some(line)
        }
    }
}

fn is_ident_char(ch: char) -> bool {
    match ch {
        '0'...'9' | 'A'...'Z' | 'a'...'z' | '_' => true,
        _ => false
    }
}

fn parse_arg_list(input: &mut CharStream) -> Result<Vec<String>,String> {
    let mut args: Vec<String> = vec![];
    input.consume_char('(');
    loop {
        match input.peek(0) {
            ',' => { input.next(); }
            ')' => { input.next(); return Ok(args) },
            ch if ch.is_whitespace() => { input.next(); }
            ch if is_ident_char(ch) => {
                args.push(parse_ident(input).to_string());
            },
            ch => return Err(format!("Unexpected char {} in macro argument list", ch))
        }
    }
}

fn parse_ident<'a>(input: &mut CharStream<'a>) -> &'a str {
    input.consume_while(|ch| is_ident_char(ch))
}

fn parse_macro(input: &mut CharStream) -> Result<CMacro,String> {
    let name = parse_ident(input);
    if name.len() == 0 {
        return Err(format!("Could not parse macro name from {}", input.tail()))
    }

    let args = if input.peek(0) == '(' {
        Some(try!(parse_arg_list(input)))
    } else {
        None
    };

    let body = input.tail().trim();

    Ok(CMacro {
        name: name.to_string(),
        args: args,
        body: if body.len() > 0 {
            Some(body.to_string())
        } else {
            None
        }
    })
}

/// Parse the source for a C header and extract
/// a list of macro definitions
pub fn extract_macros(src: &str) -> Vec<CMacro> {
    let mut macros: Vec<CMacro> = vec![];
    let line_iter = CHeaderLineIter{input: CharStream::new(src)};
    for line in line_iter {
        let mut macro_def = CharStream{input: &line, pos: 0};
        if !macro_def.consume_char('#') {
            // not a preprocessor line
            continue;
        }
        macro_def.skip_whitespace();

        if !macro_def.consume("define") || !macro_def.peek(0).is_whitespace() {
            // not a #define
            continue
        }
        macro_def.skip_whitespace();

        match parse_macro(&mut macro_def) {
            Ok(cmacro) => macros.push(cmacro),
            Err(err) => {
                panic!("failed to parse {}: {}", &line, err)
            }
        }
    }
    macros
}

/// Generates Rust source based on a set of C macro definitions and
/// a translation function which specifies how to map each macro to
/// a corresponding Rust type
pub fn generate_rust_src<TranslateFn>(defs: &[CMacro], translate_fn: TranslateFn) -> String
where TranslateFn: Fn(&CMacro) -> TranslateAction {
    let decl_lines: Vec<String> = defs.iter()
        .filter_map(|def| {
            match translate_fn(def) {
                TranslateAction::TypedConst(decl) => {
                    Some(format!("pub const {}: {} = {};", decl.name, decl.type_name, decl.expr))
                },
                TranslateAction::Skip => None
            }
        })
        .collect();
    decl_lines.connect("\n")
}

impl CMacro {
    pub fn new(name: &str, body: Option<&str>) -> CMacro {
        CMacro{ name: name.to_string(), args: None, body: body.map(|s| s.to_string())}
    }
    pub fn new_with_args(name: &str, args: Vec<&str>, body: &str) -> CMacro {
        let arg_strings: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        CMacro{ name: name.to_string(), args: Some(arg_strings), body: Some(body.to_string()) }
    }
}

/// Guess a suitable constant type for a macro
/// based on the body of the macro
pub fn guess_type(body: &str) -> &str {
    if body.starts_with("\"") {
        "&'static str"
    } else if body.contains("0x") {
        "u32"
    } else {
        "i32"
    }
}

/// Guesses a suitable translation from a C macro
/// definition to a Rust representation.
/// 
/// This is suitable for common simple cases such
/// as macros which just expand to integer or
/// string literals.
pub fn translate_macro(def: &CMacro) -> TranslateAction {
    if def.args.is_none() && def.body.is_some() {
        let body = def.body.as_ref().unwrap().clone();
        TranslateAction::TypedConst(ConstDecl{
            name: def.name.clone(),
            type_name: guess_type(&body).to_string(),
            expr: body
        })
    } else {
        TranslateAction::Skip
    }
}

#[test]
fn test_extract_macros() {
    let src = r"
#define CONST_1 1
#define CONST_2 2
#define CONST_3 3
#define NO_BODY
#define EXTRA_SPACES   4
#define MACRO_WITH_ARGS(a,b,c) ((a) + (b) + (c))
#define MACRO_WITH_ARGS_2( a , b , c ) ((a) + (b) + (c))

// commented out macros
//#define IGNORE_ME_2

#define MULTI_LINE_MACRO(a,b) \
        a + b

  #define PRECEDING_SPACES

# define SPACE_AFTER_HASH
";
    let expected_macros: Vec<CMacro> = vec![
        CMacro::new("CONST_1", Some("1")),
        CMacro::new("CONST_2", Some("2")),
        CMacro::new("CONST_3", Some("3")),
        CMacro::new("NO_BODY", None),
        CMacro::new("EXTRA_SPACES", Some("4")),
        CMacro::new_with_args("MACRO_WITH_ARGS", vec!["a","b","c"], "((a) + (b) + (c))"),
        CMacro::new_with_args("MACRO_WITH_ARGS_2", vec!["a","b","c"], "((a) + (b) + (c))"),
        CMacro::new_with_args("MULTI_LINE_MACRO", vec!["a","b"], "a + b"),
        CMacro::new("PRECEDING_SPACES", None),
        CMacro::new("SPACE_AFTER_HASH", None)
    ];
    let actual_macros = extract_macros(src);

    let expected_macro_names: Vec<&str> = expected_macros.iter().map(|m| &m.name[..]).collect();
    let actual_macro_names: Vec<&str> = actual_macros.iter().map(|m| &m.name[..]).collect();

    assert_eq!(expected_macro_names, actual_macro_names);
    for (actual, expected) in expected_macros.iter().zip(actual_macros.iter()) {
        assert_eq!(actual, expected);
    }
}

#[test]
fn test_generate_rust_src() {
    let macros: Vec<CMacro> = vec![
        CMacro::new("USED_CONST", Some("1")),
        CMacro::new("USED_CONST_2", Some("2")),
        CMacro::new("SKIPPED_CONST", Some("3"))
    ];
    let src = generate_rust_src(&macros, |ref def| {
        if def.name.starts_with("USED") {
            TranslateAction::TypedConst(ConstDecl{
                name: def.name.clone(),
                type_name: guess_type(&def.body.as_ref().unwrap()).to_string(),
                expr: def.body.as_ref().unwrap().clone()
            })
        } else {
            TranslateAction::Skip
        }
    });
    assert_eq!(src, vec![
        "pub const USED_CONST: i32 = 1;",
        "pub const USED_CONST_2: i32 = 2;"
    ].connect("\n"))
}
