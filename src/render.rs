use compile::{Statement, Template};
use context::Context;
use error::{Error, Result};
use path::Path;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf as FilePathBuf;

//TODO: Documentation.
//TODO: Benchmarks.

#[derive(Debug)]
pub struct Tenjin {
    templates: HashMap<String, Template>,
}

impl Tenjin {
    pub fn new(path: &mut FilePathBuf) -> Result<Tenjin> {
        fn recurse(
            path: &mut FilePathBuf,
            buf: &mut String,
            tenjin: &mut Tenjin,
            skip: usize,
        ) -> Result<()> {
            if path.is_dir() {
                for entry in fs::read_dir(&path)?.flat_map(|x| x.ok()) {
                    path.push(entry.file_name());
                    recurse(path, buf, tenjin, skip)?;
                    path.pop();
                }
            } else if path.extension() == Some("html".as_ref()) {
                let mut parts = path.components();

                for _ in 0..skip {
                    let _ = parts.next();
                }

                let mut name = parts.as_path().to_string_lossy().into_owned();

                let new_len = name.len().saturating_sub(5);
                name.truncate(new_len);

                buf.clear();
                File::open(&path)?.read_to_string(buf)?;
                let template = Template::compile(buf)?;
                tenjin.register(name, template);
            }

            Ok(())
        }

        let mut tenjin = Tenjin::empty();
        let mut buffer = String::new();
        let skip = path.components().count();

        recurse(path, &mut buffer, &mut tenjin, skip)?;

        Ok(tenjin)
    }

    pub fn empty() -> Tenjin {
        Tenjin {
            templates: HashMap::new(),
        }
    }

    pub fn register<S: Into<String>>(&mut self, name: S, template: Template) -> Option<Template> {
        self.templates.insert(name.into(), template)
    }

    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    pub fn render<W: Write>(
        &self,
        template: &Template,
        context: &dyn Context<W>,
        sink: &mut W,
    ) -> Result<()> {
        use self::Statement::*;

        for statement in template.body() {
            match *statement {
                Cond {
                    ref pred,
                    ref then,
                    ref otherwise,
                } => {
                    if context.truthy(Path::new(&pred)) {
                        self.render(then, context, sink)?;
                    } else if let Some(ref otherwise) = *otherwise {
                        self.render(otherwise, context, sink)?;
                    } else {
                        // No else block.
                    }
                }
                For {
                    ref ident,
                    ref path,
                    ref body,
                } => {
                    context.iterate(
                        Path::new(path),
                        Chomp {
                            caller: self,
                            body,
                            context,
                            ident,
                            sink,
                        },
                    )?;
                }
                Include {
                    ref template,
                    context: ref next,
                } => {
                    if let Some(template) = self.templates.get(template) {
                        match *next {
                            Some(ref next) => self.render(
                                template,
                                &IncludeContext {
                                    inner: context,
                                    path: next,
                                },
                                sink,
                            ),
                            None => self.render(template, context, sink),
                        }?;
                    } else {
                        return Err(Error::TemplateNotFound(template.clone()));
                    }
                }
                Inject { ref path } => {
                    context.inject(Path::new(path), sink)?;
                }
                Content { ref content } => {
                    sink.write_all(content.as_bytes())?;
                }
            }
        }

        Ok(())
    }
}

pub struct Chomp<'a, W: 'a> {
    caller: &'a Tenjin,
    body: &'a Template,
    context: &'a dyn Context<W>,
    ident: &'a str,
    sink: &'a mut W,
}

struct IncludeContext<'a, W: 'a> {
    inner: &'a dyn Context<W>,
    path: &'a str,
}

struct ForContext<'a, W: 'a> {
    back: &'a dyn Context<W>,
    front: &'a dyn Context<W>,
    name: &'a str,
}

impl<'a, W: Write> Chomp<'a, W> {
    pub fn chomp<C: Context<W>>(&mut self, item: C) -> Result<()> {
        self.caller.render(
            self.body,
            &ForContext {
                back: self.context,
                front: item.borrow(),
                name: self.ident,
            },
            self.sink,
        )
    }
}

impl<'a, W> Context<W> for IncludeContext<'a, W> {
    fn truthy(&self, path: Path) -> bool {
        let path = path.prepend(self.path);
        self.inner.truthy(path)
    }

    fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
        let path = path.prepend(self.path);
        self.inner.inject(path, sink)
    }

    fn iterate(&self, path: Path, cb: Chomp<W>) -> Result<()> {
        let path = path.prepend(self.path);
        self.inner.iterate(path, cb)
    }
}

impl<'a, W> Context<W> for ForContext<'a, W> {
    fn truthy(&self, path: Path) -> bool {
        let mut parts = path.parts();
        if parts.next() == Some(self.name) {
            self.front.truthy(parts.as_path())
        } else {
            self.back.truthy(path)
        }
    }

    fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
        let mut parts = path.parts();
        if parts.next() == Some(self.name.as_ref()) {
            self.front.inject(parts.as_path(), sink)
        } else {
            self.back.inject(path, sink)
        }
    }

    fn iterate(&self, path: Path, cb: Chomp<W>) -> Result<()> {
        let mut parts = path.parts();
        if parts.next() == Some(self.name) {
            self.front.iterate(parts.as_path(), cb)
        } else {
            self.back.iterate(path, cb)
        }
    }
}
