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
// block := { text | '{' stmt '}' }
// stmt  := for | incl | var
// for   := ident 'in' ident '}' block '{' 'end'
// incl  := 'include' ident [ 'with' ident ]
// var   := ident \ 'for' | 'include'
//
// text  := { char }
// ident := { char } \ { char } ' ' { char }

impl Template {
    pub fn compile(src: &str) -> Result<Template, Error> {
        let mut body = Vec::new();
        let mut lex  = Lexer::new(src);
    
        lex.next()?;

        while lex.peek().is_some() {
            match lex.next()? {
                Some(Symbol::Text(s)) => {
                    body.push(Statement::Content {
                        content: s.to_owned()
                    });
                }
                Some(Symbol::Open) => {
                    body.push(stmt(&mut lex)?);
                    match lex.next()? {
                        Some(Symbol::Close) => (),
                        x => return unexpected("'}'", x)
                    }
                }
                x => return unexpected("text or '{'", x)
            }
        }

        Ok(Template { body })
    }

    pub fn body(&self) -> &[Statement] {
        &*self.body
    }
}

fn stmt(lex: &mut Lexer) -> Result<Statement, Error> {
    //TODO: Distinguish between identifiers and paths.
    match lex.next()? {
        Some(Symbol::Text("for")) => {
            let mut body = Vec::new();

            let ident = match lex.next()? {
                Some(Symbol::Text(s)) => s.into(),
                x => return unexpected("identifier", x)
            };

            match lex.next()? {
                Some(Symbol::Text("in")) => (),
                x => return unexpected("'in'", x)
            }

            let path = match lex.next()? {
                Some(Symbol::Text(s)) => s.into(),
                x => return unexpected("path", x)
            };

            match lex.next()? {
                Some(Symbol::Close) => (),
                x => return unexpected("'}'", x)
            }

            loop {
                match lex.next()? {
                    Some(Symbol::Text(s)) => {
                        body.push(Statement::Content {
                            content: s.into(),
                        });
                    }
                    Some(Symbol::Open) => {
                        if lex.peek() == Some(&Symbol::Text("end")) {
                            lex.next()?;
                            break;
                        } else {
                            body.push(stmt(lex)?);
                            match lex.next()? {
                                Some(Symbol::Close) => (),
                                x => return unexpected("'}'", x)
                            }
                        }
                    }
                    x => return unexpected("text or '{'", x)
                }
            }

            let body = Template { body };
            Ok(Statement::For { ident, path, body })
        }
        Some(Symbol::Text("include")) => {
            let template = match lex.next()? {
                Some(Symbol::Text(s)) => s.into(),
                x => return unexpected("identifier", x)
            };
           
            let context = if lex.peek() == Some(&Symbol::Text("with")) {
                lex.next()?;

                match lex.next()? {
                    Some(Symbol::Text(s)) => Some(s.into()),
                    x => return unexpected("path", x)
                }
            } else {
                None
            };

            Ok(Statement::Include { template, context })
        }
        Some(Symbol::Text(s)) => {
            let path = s.into();
            Ok(Statement::Inject { path })
        }
        x => return unexpected("'for', 'include' or path", x)
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Symbol<'a> {
    Open,
    Close,
    Text(&'a str),
}

struct Lexer<'a> {
    src: &'a str,
    txt: bool,
    cur: Option<Symbol<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Lexer<'a> {
        Lexer {
            src,
            txt: true,
            cur: None,
        }
    }

    fn peek(&self) -> Option<&Symbol<'a>> {
        self.cur.as_ref()
    }

    fn next(&mut self) -> Result<Option<Symbol<'a>>, Error> {
        let mut next = if self.src.is_empty() {
            None
        } else if self.txt {
            Some(if self.src.starts_with("{{") {
                self.src = &self.src[2..];
                Symbol::Text("{")
            } else if self.src.starts_with("}}") {
                self.src = &self.src[2..];
                Symbol::Text("}")
            } else if self.src.starts_with("{") {
                self.src = &self.src[1..];
                self.txt = false;
                Symbol::Open
            } else if self.src.starts_with("}") {
                self.src = &self.src[1..];
                Symbol::Close
            } else {
                let i = self.src.find(['{', '}'].as_ref())
                    .unwrap_or(self.src.len());
                let (text, rest) = self.src.split_at(i);
                self.src = rest;
                Symbol::Text(text)
            })
        } else {
            self.src = self.src.trim_left();

            Some(if self.src.starts_with("{") {
                self.src = &self.src[1..];
                Symbol::Open
            } else if self.src.starts_with("}") {
                self.src = &self.src[1..];
                self.txt = true;
                Symbol::Close
            } else {
                let i = self.src.find(['{', '}', ' '].as_ref())
                    .unwrap_or(self.src.len());
                let (text, rest) = self.src.split_at(i);
                self.src = rest;
                Symbol::Text(text)
            })
        };

        mem::swap(&mut next, &mut self.cur);
        Ok(next)
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
