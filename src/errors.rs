use std::error;
use std::fmt;

/// Error type which is returned when an insert operation
/// does not succeed due to the underlaying buffer being exhausted.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OomError;

impl fmt::Display for OomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Out of Memory Error")
    }
}

impl error::Error for OomError {}