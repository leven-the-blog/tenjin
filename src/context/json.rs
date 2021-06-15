use context::Context;
use error::{Error, Result};
use path::Path;
use render::Chomp;
use serde_json::Value;
use std::io::Write;

impl<W: Write> Context<W> for Value {
    fn truthy(&self, path: Path) -> bool {
        use self::Value::*;

        let mut value = self;

        for part in path.parts() {
            if let Some(next_value) = value.get(part) {
                value = next_value;
            } else {
                // Undefined is falsey.
                return false;
            }
        }

        match *value {
            Null => false,
            Bool(b) => b,
            Number(ref n) => n.as_f64() != Some(0.0),
            String(ref s) => !s.is_empty(),
            Array(_) | Object(_) => true,
        }
    }

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
            Null => {
                sink.write_all(b"null")?;
            }

            Bool(b) => {
                sink.write_all(if b { b"true" } else { b"false" })?;
            }

            Number(ref n) => {
                write!(sink, "{}", n)?;
            }

            String(ref s) => {
                s.inject(Path::new(""), sink)?;
            }

            Array(_) | Object(_) => {
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

        if let Value::Array(ref array) = *value {
            for value in array {
                chomp.chomp(value)?;
            }
            Ok(())
        } else {
            Err(Error::NotIterable(path.to_owned()))
        }
    }
}
