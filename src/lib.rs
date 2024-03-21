pub use proc::CVarEnum;

use std::{
    num::{
        IntErrorKind, NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize,
        NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, ParseIntError,
    },
    sync::{Arc, RwLock},
};

// Valid types
// - bool
// - u*
// - i*
// - usize
// - isize
// - f32
// - f64
// - String
// - User Defined Enum
//
// Constraints
// - String
//  - Specific values, eg. "slow" "normal" "fast" (custom enums could be used as a replacement?)
//  - Min and max length, eg. 4-24 characters for username
// - Integer
//  - Specific range eg. 512..=2048
//
// Syntax
// CVar
// [<namespace>.]<name> <value>
//
// r.full_bright <enabled:bool>
// Disables all lighting when enabled.
//
// map <name:string> [gamemode:string]
// Changes the map to map with matching name with optional gamemode specified.

#[derive(Clone)]
pub struct CVar<T: Value>(Arc<InnerCVar<T>>);

impl<T: Value> CVar<T> {}

pub struct InnerCVar<T: Value> {
    name: &'static str,
    description: &'static str,
    value: RwLock<T>,
}

pub trait Value: Sized {
    fn parse(s: &str) -> Result<Self, Error>;
    fn validate(s: &str) -> Result<Vec<String>, Error>;
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue {
        value: String,
    },
    EmptyValue,
    TooBig {
        value: String,
        min: String,
        max: String,
    },
    TooSmall {
        value: String,
        min: String,
        max: String,
    },
}

impl Error {
    #[inline]
    pub fn invalid_value(value: &str) -> Self {
        Self::InvalidValue {
            value: value.to_string(),
        }
    }

    #[inline]
    pub fn too_large(value: &str, min: &str, max: &str) -> Self {
        Self::TooBig {
            value: value.to_string(),
            min: min.to_string(),
            max: max.to_string(),
        }
    }

    #[inline]
    pub fn too_small(value: &str, min: &str, max: &str) -> Self {
        Self::TooSmall {
            value: value.to_string(),
            min: min.to_string(),
            max: max.to_string(),
        }
    }

    pub fn from_parse_int_error(e: ParseIntError, value: &str, min: &str, max: &str) -> Self {
        match e.kind() {
            IntErrorKind::Empty => Self::EmptyValue,
            IntErrorKind::InvalidDigit => Self::invalid_value(value),
            IntErrorKind::PosOverflow => Self::too_large(value, min, max),
            IntErrorKind::NegOverflow => Self::too_small(value, min, max),
            IntErrorKind::Zero => Self::invalid_value(value),
            _ => unreachable!("match should be exhaustive but rust-analyzer doesn't recognize it as such ¯\\_(ツ)_/¯"),
        }
    }
}

// impl From<ParseFloatError> for Error {
//     fn from(_value: ParseFloatError) -> Self {
//         Self::InvalidValue
//     }
// }

impl Value for bool {
    fn parse(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_str() {
            "t" => Ok(true),
            "f" => Ok(false),
            "true" => Ok(true),
            "false" => Ok(false),
            "1" => Ok(true),
            "0" => Ok(false),
            _ => Err(Error::invalid_value(s)),
        }
    }

    fn validate(s: &str) -> Result<Vec<String>, Error> {
        const VALUES: [&str; 2] = ["true", "false"];

        let mut values = vec![];

        for value in VALUES {
            if value.starts_with(s) {
                values.push(value.to_string());
            }
        }

        Ok(values)
    }
}

impl Value for String {
    fn parse(s: &str) -> Result<Self, Error> {
        if s.starts_with("\"") && s.ends_with("\"") {
            Ok(s[1..s.len() - 1].to_string())
        } else {
            Err(Error::invalid_value(s))
        }
    }

    fn validate(s: &str) -> Result<Vec<String>, Error> {
        if s.starts_with("\"") && s.ends_with("\"") {
            Ok(vec![])
        } else {
            Err(Error::invalid_value(s))
        }
    }
}

macro_rules! impl_value_int {
    ($error_fn:expr, $($t:ty),+ $(,)?) => {
        $(
            impl Value for $t {
                fn parse(s: &str) -> Result<Self, Error> {
                    let v = s.parse::<$t>().map_err(|e| $error_fn(e, s, &<$t>::MIN.to_string(), &<$t>::MAX.to_string()))?;

                    Ok(v)
                }

                fn validate(s: &str) -> Result<Vec<String>, Error> {
                    let _ = s.parse::<$t>().map_err(|e| $error_fn(e, s, &<$t>::MIN.to_string(), &<$t>::MAX.to_string()))?;

                    Ok(vec![])
                }
            }
        )+
    };
}

macro_rules! impl_value_float {
    ($($t:ty),+ $(,)?) => {
        $(
            impl Value for $t {
                fn parse(s: &str) -> Result<Self, Error> {
                    let v = s.parse::<$t>().map_err(|_e| Error::invalid_value(s))?;

                    Ok(v)
                }

                fn validate(s: &str) -> Result<Vec<String>, Error> {
                    let _ = s.parse::<$t>().map_err(|_e| Error::invalid_value(s))?;

                    Ok(vec![])
                }
            }
        )+
    };
}

// Integers
impl_value_int!(
    Error::from_parse_int_error,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    usize,
    isize
);

// Non-Zero Integers
impl_value_int!(
    Error::from_parse_int_error,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroUsize,
    NonZeroIsize
);

// Floating Point Numbers
impl_value_float!(f32, f64);
