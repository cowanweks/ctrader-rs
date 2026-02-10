mod client;
mod error;
mod types;

pub mod openapi;

pub mod prelude {
    pub use super::client::traits::*;
    pub use super::openapi::{self};
    pub use super::types::*;
}
