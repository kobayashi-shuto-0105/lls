mod proposal;
mod safety;
mod writer;

pub use proposal::generate_proposal;
pub use safety::safety_check;
pub use writer::{WriteResult, write_config};
