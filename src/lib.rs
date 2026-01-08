pub mod lychrel;
pub mod search;
pub mod verify;
pub mod checkpoint;

pub use lychrel::{is_palindrome, lychrel_iteration, reverse_number, IterationResult};
pub use search::{search_range, SearchConfig, SearchResults};
pub use verify::{verify_lychrel, verify_lychrel_resumable, resume_from_checkpoint, resume_from_checkpoint_with_config, VerifyConfig, VerifyResult};
pub use checkpoint::Checkpoint;
