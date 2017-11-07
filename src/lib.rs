extern crate htmlescape;
extern crate serde_json;

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
