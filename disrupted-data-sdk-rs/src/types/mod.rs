pub mod error;
pub mod actions;
pub mod identity;

pub use actions::GetRequest;
pub use actions::PutRequest;
pub use error::DisruptedDataError;
pub use identity::Identity;
use sha2::digest::Update;
use sha2::Digest;
use std::io::Write;


