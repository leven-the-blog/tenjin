extern crate html_escape;
#[cfg(feature = "serde_json")]
extern crate serde_json;
#[cfg(feature = "toml")]
extern crate toml;

pub mod compile;
pub mod context;
pub mod error;
mod macros;
pub mod path;
pub mod render;

pub use compile::Template;
pub use context::{Context, Raw};
pub use error::{Error, Result};
pub use render::Tenjin;
