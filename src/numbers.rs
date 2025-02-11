use lexical_parse_float::{format as lexical_format, FromLexicalWithOptions, Options as ParseFloatOptions};

/// Parse a string as an int.
///
/// This is around 2x faster than using `str::parse::<i64>()`
pub fn int_parse_str(s: &str) -> Option<i64> {
    int_parse_bytes(s.as_bytes())
}

/// Parse bytes as an int.
pub fn int_parse_bytes(s: &[u8]) -> Option<i64> {
    let (neg, first_digit, digits) = match s {
        [b'-', first, digits @ ..] => (true, first, digits),
        [b'+', first, digits @ ..] | [first, digits @ ..] => (false, first, digits),
        _ => return None,
    };

    let mut result = match first_digit {
        b'0' => 0,
        b'1'..=b'9' => (first_digit & 0x0f) as i64,
        _ => return None,
    };

    for digit in digits {
        result = result.checked_mul(10)?;
        match digit {
            b'0' => {}
            b'1'..=b'9' => result = result.checked_add((digit & 0x0f) as i64)?,
            _ => return None,
        }
    }
    if neg {
        Some(-result)
    } else {
        Some(result)
    }
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
    let (neg, first_digit, digits) = match s {
        [b'-', first, digits @ ..] => (true, *first, digits),
        [b'+', first, digits @ ..] | [first, digits @ ..] => (false, *first, digits),
        [] => return IntFloat::Err,
    };

    let int_part = DECODE_MAP[first_digit as usize];

    if int_part == ERR {
        return IntFloat::Err;
    }

    let mut int_part = int_part as i64;

    for &digit in digits {
        let value = DECODE_MAP[digit as usize];

        debug_assert!(value <= 9 || value == ERR);

        if value == ERR {
            return parse_possible_float(s, digit);
        }

        int_part = int_part.wrapping_mul(10);
        int_part = int_part.wrapping_add(value as i64);

        // if overflow occurred, return an error
        if int_part < 0 {
            return IntFloat::Err;
        }
    }

    if neg {
        IntFloat::Int(-int_part)
    } else {
        IntFloat::Int(int_part)
    }
}

const ERR: u8 = 0xff;

#[rustfmt::skip]
static DECODE_MAP: [u8; 256] = {
    const __: u8 = ERR;
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
         0,  1,  2,  3,  4,  5,  6,  7,  8,  9, __, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

#[inline(never)] // avoid making the hot loop large
fn parse_possible_float(s: &[u8], current_digit: u8) -> IntFloat {
    if current_digit != b'.' {
        return IntFloat::Err;
    }

    static OPTIONS: ParseFloatOptions = ParseFloatOptions::new();
    return match f64::from_lexical_with_options::<{ lexical_format::STANDARD }>(s, &OPTIONS) {
        Ok(v) => IntFloat::Float(v),
        Err(_) => IntFloat::Err,
    };
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
