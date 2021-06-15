use error::{Error, Result};
use path::Path;
use render::Chomp;
use html_escape;
use std::borrow::Borrow;
use std::io::Write;

#[cfg(feature = "serde_json")]
mod json;
#[cfg(feature = "toml")]
mod toml;

pub trait Context<W> {
    fn truthy(&self, path: Path) -> bool;
    fn inject(&self, path: Path, sink: &mut W) -> Result<()>;
    fn iterate(&self, path: Path, chomp: Chomp<W>) -> Result<()>;
}

impl<'a, W, T: Context<W> + ?Sized> Context<W> for &'a T {
    fn truthy(&self, path: Path) -> bool {
        (*self).truthy(path)
    }

    fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
        (*self).inject(path, sink)
    }

    fn iterate(&self, path: Path, chomp: Chomp<W>) -> Result<()> {
        (*self).iterate(path, chomp)
    }
}

impl<W: Write, C: Context<W>> Context<W> for Option<C> {
    fn truthy(&self, path: Path) -> bool {
        if let Some(ref c) = *self {
            c.truthy(path)
        } else {
            false
        }
    }

    fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
        if let Some(ref c) = *self {
            c.inject(path, sink)
        } else {
            Err(Error::Undefined(path.to_owned()))
        }
    }

    fn iterate(&self, path: Path, chomp: Chomp<W>) -> Result<()> {
        if let Some(ref c) = *self {
            c.iterate(path, chomp)
        } else {
            Err(Error::Undefined(path.to_owned()))
        }
    }
}

impl<W: Write> Context<W> for str {
    fn truthy(&self, path: Path) -> bool {
        match path.parts().next() {
            Some(_) => false,
            None => !self.is_empty(),
        }
    }

    fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
        match path.parts().next() {
            Some(_) => Err(Error::Undefined(path.to_owned())),
            None => {
                //TODO: Remember to warn people to only inject into content positions.
                html_escape::encode_text_minimal_to_writer(self, sink)?;
                // htmlescape::encode_minimal_w(self, sink)?;
                Ok(())
            },
        }
    }

    fn iterate(&self, path: Path, _: Chomp<W>) -> Result<()> {
        match path.parts().next() {
            Some(_) => Err(Error::Undefined(path.to_owned())),
            None => Err(Error::NotIterable(path.to_owned())),
        }
    }
}

#[derive(Debug)]
pub struct Raw<S>(pub S);

impl<W, S> Context<W> for Raw<S>
where
    S: Borrow<str>,
    W: Write,
{
    fn truthy(&self, path: Path) -> bool {
        let s = self.0.borrow();
        <str as Context<W>>::truthy(s, path)
    }

    fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
        match path.parts().next() {
            Some(_) => Err(Error::Undefined(path.to_owned())),
            None => {
                sink.write_all(self.0.borrow().as_bytes())?;
                Ok(())
            },
        }
    }

    fn iterate(&self, path: Path, chomp: Chomp<W>) -> Result<()> {
        self.0.borrow().iterate(path, chomp)
    }
}

impl<W: Write> Context<W> for bool {
    fn truthy(&self, path: Path) -> bool {
        match path.parts().next() {
            Some(_) => false,
            None => *self,
        }
    }

    fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
        match path.parts().next() {
            Some(_) => Err(Error::Undefined(path.to_owned())),
            None => {
                sink.write_all(if *self { b"true" } else { b"false" })?;
                Ok(())
            },
        }
    }

    fn iterate(&self, path: Path, _: Chomp<W>) -> Result<()> {
        match path.parts().next() {
            Some(_) => Err(Error::Undefined(path.to_owned())),
            None => Err(Error::NotIterable(path.to_owned())),
        }
    }
}

macro_rules! num_impl {
($x:ty, $y:expr) => {
    impl<W: Write> Context<W> for $x {
        fn truthy(&self, path: Path) -> bool {
            match path.parts().next() {
                Some(_) => false,
                None => *self != $y,
            }
        }

        fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
            match path.parts().next() {
                Some(_) => Err(Error::Undefined(path.to_owned())),
                None => {
                    write!(sink, "{}", self)?;
                    Ok(())
                },
            }
        }

        fn iterate(&self, path: Path, _: Chomp<W>) -> Result<()> {
            match path.parts().next() {
                Some(_) => Err(Error::Undefined(path.to_owned())),
                None => Err(Error::NotIterable(path.to_owned())),
            }
        }
    }
}
}

macro_rules! int_impls {
( $($x:ty,)* ) => {
    $(num_impl!($x, 0);)*
}
}

macro_rules! float_impls {
( $($x:ty,)* ) => {
    $(num_impl!($x, 0.0);)*
}
}

int_impls! {
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
}

float_impls! {
    f32, f64,
}
