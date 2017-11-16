use context::Context;
use error::{Error, Result};
use path::Path;
use render::Chomp;
use toml::Value;
use std::io::Write;

impl<W: Write> Context<W> for Value {
    fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
        use self::Value::*;

        let mut value = self;

        for part in path.parts() {
            if let Some(next_value) = value.get(part) {
                value = next_value;
            } else {
                return Err(Error::Undefined(path.to_owned()));
            }
        }

        match *value {
            String(ref s) => {
                s.inject(Path::new(""), sink)?;
            }

            Integer(n) => {
                write!(sink, "{}", n)?;
            }

            Float(n) => {
                write!(sink, "{}", n)?;
            }

            Boolean(b) => {
                sink.write_all(if b { b"true" } else { b"false" })?;
            }

            Datetime(ref date) => {
                write!(sink, "{}", date)?;
            }

            Array(_) | Table(_) => {
                return Err(Error::NotInjectable(path.to_owned()));
            }
        }

        Ok(())
    }

    fn iterate(&self, path: Path, mut chomp: Chomp<W>) -> Result<()> {
        let mut value = self;

        for part in path.parts() {
            if let Some(next_value) = value.get(part) {
                value = next_value;
            } else {
                return Err(Error::Undefined(path.to_owned()));
            }
        }

        if let &Value::Array(ref array) = value {
            for value in array {
                chomp.chomp(value)?;
            }
            Ok(())
        } else {
            Err(Error::NotIterable(path.to_owned()))
        }
    }
}
