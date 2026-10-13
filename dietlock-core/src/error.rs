use std::fmt;

/// Input validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Date not in the calendar, or not in the strict `YYYY-MM-DD` format.
    InvalidDate(String),
    /// Value outside its allowed range.
    OutOfRange {
        field: &'static str,
        value: i64,
        min: i64,
        max: i64,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDate(s) => write!(f, "invalid date: {s:?} (expected YYYY-MM-DD)"),
            Self::OutOfRange {
                field,
                value,
                min,
                max,
            } => write!(f, "{field} = {value} out of range [{min}, {max}]"),
        }
    }
}

impl std::error::Error for ValidationError {}

pub(crate) fn check_range(
    field: &'static str,
    value: u16,
    min: u16,
    max: u16,
) -> Result<(), ValidationError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(ValidationError::OutOfRange {
            field,
            value: value.into(),
            min: min.into(),
            max: max.into(),
        })
    }
}
