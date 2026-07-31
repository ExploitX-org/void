#[derive(Debug, thiserror::Error)]
pub enum ParseError {
  #[error("{0} must not be empty")]
  Empty(&'static str),

  #[error("{0} exceeds maximum length of {1}")]
  TooLong(&'static str, usize),

  #[error("{0} must be at least {1} characters")]
  TooShort(&'static str, usize),

  #[error("{0} has an invalid format")]
  InvalidFormat(&'static str),

  #[error("{0} must be between {1} and {2} characters in length")]
  InvalidLength(&'static str, usize, usize),

  #[error("{0} must be between {1} and {2}")]
  OutOfRange(&'static str, i32, i32),

  #[error("{0} is required")]
  MissingField(&'static str),

  #[error("internal error: {0}")]
  Internal(String),
}

pub type Result<T> = std::result::Result<T, ParseError>;
