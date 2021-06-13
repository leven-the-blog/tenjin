use path::PathBuf;
use std::borrow::Borrow;
use std::error::Error as StdError;
use std::{fmt, mem};

#[derive(Debug, Clone)]
pub struct Template {
    body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Cond {
        pred: PathBuf,
        then: Template,
        otherwise: Option<Template>,
    },
    For {
        ident: String,
        path:  PathBuf,
        body:  Template,
    },
    Include {
        template: String,
        context:  Option<PathBuf>,
    },
    Inject {
        path: PathBuf,
    },
    Content {
        content: String,
    },
}

// Rules
// =====
//
// stmt  := for | cond | incl | var
// cond  := 'if' path '}' block [ '{' else '}' block ] '{' end
// for   := 'for' ident 'in' path '}' block '{' 'end'
// incl  := 'include' path [ 'with' path ]
// var   := path \ 'for' | 'include' | 'if'
//
// ident := word \ { char } '.' { char }
// path  := word
//
// text  := { char }
// word  := { char } \ { char } (' ' | '{' | '}') { char }

impl Template {
    pub fn compile(src: &str) -> Result<Template, Error> {
        let mut body = Vec::new();
        let mut lex  = Lexer::new(src);

        while let Some(sym) = lex.next() {
            match sym {
                Symbol::Text(s) => {
                    body.push(Statement::Content {
                        content: s.into(),
                    });
                }
                Symbol::Open => {
                    body.push(stmt(&mut lex)?);
                    expect(&mut lex, Symbol::Close, "'}'")?;
                }
                x => {
                    return unexpected("text or '{'", Some(x));
                }
            }
        }

        Ok(Template { body })
    }

    pub fn body(&self) -> &[Statement] {
        &*self.body
    }
}

fn stmt(lex: &mut Lexer) -> Result<Statement, Error> {
    match lex.peek() {
        Some(&Symbol::Word("if")) => cond(lex),
        Some(&Symbol::Word("for")) => forr(lex),
        Some(&Symbol::Word("include")) => incl(lex),
        Some(&Symbol::Word(_)) => var(lex),
        _ => unexpected("'if', 'for', 'include' or path", lex.peek()),
    }
}

fn cond(lex: &mut Lexer) -> Result<Statement, Error> {
    expect(lex, Symbol::Word("if"), "'if'")?;

    let pred = path(lex)?.into();

    expect(lex, Symbol::Close, "'}'")?;

    let mut then = Vec::new();
    let mut otherwise = None;

    loop {
        match lex.next() {
            Some(Symbol::Text(s)) => {
                let body = match otherwise {
                    Some(ref mut body) => body,
                    None => &mut then,
                };

                body.push(Statement::Content {
                    content: s.into(),
                });
            }
            Some(Symbol::Open) => {
                match lex.peek() {
                    Some(&Symbol::Word("end")) => {
                        let _ = lex.next();
                        break;
                    }
                    Some(&Symbol::Word("else")) => {
                        let _ = lex.next();
                        expect(lex, Symbol::Close, "'}'")?;
                        otherwise = Some(Vec::new());
                    }
                    _ => {
                        let body = match otherwise {
                            Some(ref mut body) => body,
                            None => &mut then,
                        };

                        body.push(stmt(lex)?);
                        expect(lex, Symbol::Close, "'}'")?;
                    }
                }
            }
            x => {
                return unexpected("text or '{'", x);
            }
        }
    }

    let then = Template { body: then };
    let otherwise = otherwise.map(|x| {
        Template { body: x }
    });

    Ok(Statement::Cond { pred, then, otherwise })
}

fn forr(lex: &mut Lexer) -> Result<Statement, Error> {
    expect(lex, Symbol::Word("for"), "'for'")?;

    let x = ident(lex)?.into();

    expect(lex, Symbol::Word("in"), "'in'")?;

    let y = path(lex)?.into();

    expect(lex, Symbol::Close, "'}'")?;

    let mut body = Vec::new();

    loop {
        match lex.next() {
            Some(Symbol::Text(s)) => {
                body.push(Statement::Content {
                    content: s.into(),
                });
            }
            Some(Symbol::Open) => {
                match lex.peek() {
                    Some(&Symbol::Word("end")) => {
                        let _ = lex.next();
                        break;
                    }
                    _ => {
                        body.push(stmt(lex)?);
                        expect(lex, Symbol::Close, "'}'")?;
                    }
                }
            }
            x => {
                println!("{}", lex.src);
                return unexpected("text or '{'", x);
            }
        }
    }

    let body = Template { body };
    Ok(Statement::For { ident: x, path: y, body })
}

fn incl(lex: &mut Lexer) -> Result<Statement, Error> {
    expect(lex, Symbol::Word("include"), "'include'")?;

    let x = path(lex)?.into();

    let context = if lex.peek() == Some(&Symbol::Word("with")) {
        let _ = lex.next();
        Some(path(lex)?.into())
    } else {
        None
    };

    Ok(Statement::Include { template: x, context })
}

fn var(lex: &mut Lexer) -> Result<Statement, Error> {
    let x = path(lex)?.into();
    Ok(Statement::Inject { path: x })
}

fn ident<'a>(lex: &mut Lexer<'a>) -> Result<&'a str, Error> {
    let sym = lex.next();

    if let Some(Symbol::Word(ident)) = sym {
        if !ident.contains('.') {
            return Ok(ident);
        }
    }

    unexpected("ident", sym)
}

fn path<'a>(lex: &mut Lexer<'a>) -> Result<&'a str, Error> {
    match lex.next() {
        Some(Symbol::Word(path)) => Ok(path),
        x => unexpected("path", x),
    }
}

fn expect(lex: &mut Lexer, sym: Symbol, expected: &'static str) -> Result<(), Error> {
    let found = lex.next();

    if found == Some(sym) {
        Ok(())
    } else {
        unexpected(expected, found)
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Symbol<'a> {
    // Open brace.
    Open,
    // Close brace.
    Close,
    // Raw HTML.
    Text(&'a str),
    // A word inside the braces.
    // Guaranteed not to contain a space.
    Word(&'a str),
}

struct Lexer<'a> {
    src: &'a str,
    txt: bool,
    cur: Option<Symbol<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Lexer<'a> {
        let mut res = Lexer {
            src,
            txt: true,
            cur: None,
        };

        assert_eq!(res.next(), None);
        res
    }

    fn peek(&self) -> Option<&Symbol<'a>> {
        self.cur.as_ref()
    }

    fn next(&mut self) -> Option<Symbol<'a>> {
        let mut next = if self.src.is_empty() {
            // Nothing to process.
            None
        } else if self.txt {
            // Text mode (not inside braces.)

            Some(if self.src.starts_with("{{") {
                // Escaped open brace.
                self.src = &self.src[2..];
                Symbol::Text("{")
            } else if self.src.starts_with("}}") {
                // Escaped close brace.
                self.src = &self.src[2..];
                Symbol::Text("}")
            } else if self.src.starts_with('{') {
                // Open brace (exit text mode.)
                self.src = &self.src[1..];
                self.txt = false;
                Symbol::Open
            } else if self.src.starts_with('}') {
                // Close brace (should be an error.)
                self.src = &self.src[1..];
                Symbol::Close
            } else {
                // Raw HTML upto the next brace / EOF.
                let i = self.src.find(&['{', '}'][..])
                    .unwrap_or(self.src.len());
                let (text, rest) = self.src.split_at(i);
                self.src = rest;
                Symbol::Text(text)
            })
        } else {
            // Inside braces.

            // Ignore whitespace.
            self.src = self.src.trim_start();

            Some(if self.src.starts_with("{") {
                // Another open brace (should be an error.)
                self.src = &self.src[1..];
                Symbol::Open
            } else if self.src.starts_with("}") {
                // A close brace (re-enter text mode.)
                self.src = &self.src[1..];
                self.txt = true;
                Symbol::Close
            } else {
                // Word until next whitespace / brace / EOF.
                let i = self.src.find(['{', '}', ' '].as_ref())
                    .unwrap_or(self.src.len());
                let (word, rest) = self.src.split_at(i);
                self.src = rest;
                Symbol::Word(word)
            })
        };

        mem::swap(&mut next, &mut self.cur);
        next
    }
}

#[derive(Debug)]
pub enum Error {
    Unexpected(&'static str, String),
}

fn unexpected<'a, T: Borrow<Symbol<'a>>, U>(
    expected: &'static str,
    found: Option<T>
) -> Result<U, Error>
{
    let found = match found {
        Some(found) => {
            match found.borrow() {
                &Symbol::Open    => "'{'",
                &Symbol::Close   => "'}'",
                &Symbol::Text(s) => s,
                &Symbol::Word(s) => s,
            }
        },
        None => "nothing",
    };

    Err(Error::Unexpected(expected, found.into()))
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Unexpected(_, _) => "unexpected input",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Unexpected(a, ref b) => {
                write!(f, "expected {}, found {}", a, b)
            }
        }
    }
}
