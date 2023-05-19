// exported modules
pub mod error;
pub mod model;

// client impls
pub mod atcoder;

// re-exports
pub use atcoder::AtCoderClient;
pub use error::*;
pub use model::*;

pub fn new_client(platform: Platform) -> Box<dyn Client> {
    use Platform::*;
    match platform {
        AtCoder => Box::new(AtCoderClient::new()),
    }
}

// internal modules
mod util;
