pub mod lychrel;
pub mod search;

pub use lychrel::{is_palindrome, lychrel_iteration, reverse_number, IterationResult};
pub use search::{search_range, SearchConfig, SearchResults};
