extern crate serde_json;

mod context;
mod error;
mod macros;
mod path;
mod template;

pub use context::*;
pub use error::*;
pub use path::*;
pub use template::*;

use serde_json::Value;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path as FilePath;
use std::io::{Read, Write};

//TODO: Conditionals.
//TODO: Documentation.
//TODO: Use tendrils to eliminate allocations during compilation.
//TODO: Benchmarks.

pub struct Tenjin {
    templates: HashMap<String, Template>,
}

impl Tenjin {
    pub fn new<I>(paths: I) -> Result<Tenjin>
    where
        I: IntoIterator,
        I::Item: AsRef<FilePath>,
    {
        let mut tenjin = Tenjin::empty();
        let mut buffer = String::new();

        for path in paths {
            let name = match path.as_ref().file_stem() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => continue,
            };

            buffer.clear();
            File::open(path)?.read_to_string(&mut buffer)?;
            let template = Template::compile(&buffer)?;
            tenjin.register(name, template);
        }

        Ok(tenjin)
    }

    pub fn empty() -> Tenjin {
        Tenjin {
            templates: HashMap::new(),
        }
    }

    pub fn register<S: Into<String>>(
        &mut self,
        name: S,
        template: Template
    ) -> Option<Template> {
        self.templates.insert(name.into(), template)
    }

    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    pub fn render<W: Write>(
        &self,
        template: &Template,
        context: &Context<W>,
        sink: &mut W,
    ) -> Result<()> {
        use self::Statement::*;

        for statement in template.body() {
            match statement {
                &For { ref ident, ref path, ref body } => {
                    context.iterate(Path::new(path), Chomp {
                        caller: self,
                        body: body,
                        context: context,
                        ident: ident,
                        sink: sink,
                    })?;
                },
                &Include { ref template, context: ref next } => {
                    if let Some(template) = self.templates.get(template) {
                        match next {
                            &Some(ref next) => self.render(
                                template,
                                &IncludeContext {
                                    inner: context,
                                    path: next,
                                },
                                sink,
                            ),
                            &None => self.render(
                                template,
                                context,
                                sink,
                            ),
                        }?;
                    } else {
                        return Err(Error::TemplateNotFound(template.clone()));
                    }
                },
                &Inject { ref path } => {
                    context.inject(Path::new(path), sink)?;
                },
                &Content { ref content } => {
                    sink.write_all(content.as_bytes())?;
                },
            }
        }

        Ok(())
    }
}

pub struct Chomp<'a, W: 'a> {
    caller: &'a Tenjin,
    body: &'a Template,
    context: &'a Context<W>,
    ident: &'a str,
    sink: &'a mut W,
}

struct IncludeContext<'a, W: 'a> {
    inner: &'a Context<W>,
    path: &'a str,
}

struct ForContext<'a, W: 'a> {
    back: &'a Context<W>,
    front: &'a Context<W>,
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
    fn inject(
        &self,
        path: Path,
        sink: &mut W
    ) -> Result<()> {
        let path = path.prepend(self.path);
        self.inner.inject(path, sink)
    }

    fn iterate(
        &self,
        path: Path,
        cb: Chomp<W>
    ) -> Result<()> {
        let path = path.prepend(self.path);
        self.inner.iterate(path, cb)
    }
}

impl<'a, W> Context<W> for ForContext<'a, W> {
    fn inject(
        &self,
        path: Path,
        sink: &mut W
    ) -> Result<()> {
        let mut parts = path.parts();
        if parts.next() == Some(self.name.as_ref()) {
            self.front.inject(parts.as_path(), sink)
        } else {
            self.back.inject(path, sink)
        }
    }

    fn iterate(
        &self,
        path: Path,
        cb: Chomp<W>
    ) -> Result<()> {
        let mut parts = path.parts();
        if parts.next() == Some(self.name.as_ref()) {
            self.front.iterate(parts.as_path(), cb)
        } else {
            self.back.iterate(path, cb)
        }
    }
}
