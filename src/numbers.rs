use lexical_parse_float::{FromLexicalWithOptions, Options as ParseFloatOptions, format as lexical_format};

/// Parse a string as an int.
///
/// This is around 2x faster than using `str::parse::<i64>()`
pub fn int_parse_str(s: &str) -> Option<i64> {
    int_parse_bytes(s.as_bytes())
}

/// Parse bytes as an int.
pub fn int_parse_bytes(s: &[u8]) -> Option<i64> {
    int_parse_bytes_internal(s).ok()
}

#[derive(Debug)]
pub enum IntFloat {
    Int(i64),
    Float(f64),
    Err,
}

impl IntFloat {
    pub fn is_err(&self) -> bool {
        matches!(self, IntFloat::Err)
    }
}

/// Parse a string as a float.
///
/// This is around 2x faster than using `str::parse::<f64>()`
pub fn float_parse_str(s: &str) -> IntFloat {
    float_parse_bytes(s.as_bytes())
}

/// Parse bytes as an float.
pub fn float_parse_bytes(s: &[u8]) -> IntFloat {
    // optimistically try to parse as an integer
    match int_parse_bytes_internal(s) {
        Ok(int) => return IntFloat::Int(int),
        // integer parsing failed on encountering a '.', fall through to try as a float
        Err(IntParseError::InvalidDigit(b'.')) => {}
        // alternatively long floats might overflow the integer parser
        Err(IntParseError::Overflow) if s.contains(&b'.') => {}
        // any other integer parse error is also a float error
        Err(_) => return IntFloat::Err,
    }

    static OPTIONS: ParseFloatOptions = ParseFloatOptions::new();
    match f64::from_lexical_with_options::<{ lexical_format::STANDARD }>(s, &OPTIONS) {
        Ok(v) => IntFloat::Float(v),
        Err(_) => IntFloat::Err,
    }
}

enum IntParseError {
    InvalidDigit(u8),
    Overflow,
    Empty,
}

/// Optimized routine to either parse an integer or return the character which triggered the error.
fn int_parse_bytes_internal(s: &[u8]) -> Result<i64, IntParseError> {
    let (neg, first_digit, digits) = match s {
        [b'-', first, digits @ ..] => (true, *first, digits),
        [b'+', first, digits @ ..] | [first, digits @ ..] => (false, *first, digits),
        [] => return Err(IntParseError::Empty),
    };

    // accumulate as u64 so that i64::MIN can be represented
    let mut int_part = decoded_u64_value(first_digit)?;

    let (digits_short, digits_long) = digits.split_at_checked(18).unwrap_or((digits, &[]));

    // for first 19 digits can use simple loop with no risk of overflow
    // (one already parsed, do up to 18 more here)
    for &digit in digits_short {
        let value = decoded_u64_value(digit)?;
        int_part = int_part.wrapping_mul(10);
        int_part = int_part.wrapping_add(value);
    }

    for &digit in digits_long {
        let value = decoded_u64_value(digit)?;
        int_part = int_part.checked_mul(10).ok_or(IntParseError::Overflow)?;
        int_part = int_part.checked_add(value).ok_or(IntParseError::Overflow)?;
    }

    Ok(if neg {
        const LIMIT: u64 = i64::MIN.unsigned_abs();
        if int_part > LIMIT {
            return Err(IntParseError::Overflow);
        }
        int_part.wrapping_neg().cast_signed()
    } else {
        const LIMIT: u64 = i64::MAX.unsigned_abs();
        if int_part > LIMIT {
            return Err(IntParseError::Overflow);
        }
        int_part.cast_signed()
    })
}

/// Helper to parse a single ascii digit as an i64.
fn decoded_u64_value(digit: u8) -> Result<u64, IntParseError> {
    let decoded = digit.wrapping_sub(b'0');
    if decoded > 9 {
        return Err(IntParseError::InvalidDigit(digit));
    }
    Ok(decoded as u64)
}

/// Count the number of decimal places in a byte slice.
/// Caution: does not verify the integrity of the input,
/// so it may return incorrect results for invalid inputs.
pub(crate) fn decimal_digits(bytes: &[u8]) -> usize {
    match bytes.splitn(2, |&b| b == b'.').nth(1) {
        Some(b"") | None => 0,
        Some(fraction) => fraction.len(),
    }
}
