extern crate htmlescape;
#[cfg(feature = "serde_json")]
extern crate serde_json;
#[cfg(feature = "toml")]
extern crate toml;

mod macros;
pub mod compile;
pub mod context;
pub mod error;
pub mod path;
pub mod render;

pub use context::{Raw, Context};
pub use error::{Error, Result};
pub use render::Tenjin;
pub use compile::Template;
