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
        [b'-', first, digits @ ..] => (true, first, digits),
        [b'+', first, digits @ ..] | [first, digits @ ..] => (false, first, digits),
        _ => return IntFloat::Err,
    };

    let mut int_part = match first_digit {
        b'0' => 0,
        b'1'..=b'9' => (first_digit & 0x0f) as i64,
        _ => return IntFloat::Err,
    };

    let mut found_dot = false;

    let mut bytes = digits.iter().copied();

    for digit in bytes.by_ref() {
        match digit {
            b'0'..=b'9' => {
                int_part = match int_part.checked_mul(10) {
                    Some(i) => i,
                    None => return IntFloat::Err,
                };
                int_part = match int_part.checked_add((digit & 0x0f) as i64) {
                    Some(i) => i,
                    None => return IntFloat::Err,
                };
            }
            b'.' => {
                found_dot = true;
                break;
            }
            _ => return IntFloat::Err,
        }
    }

    if found_dot {
        let mut result = int_part as f64;
        let mut div = 10_f64;
        for digit in bytes {
            match digit {
                b'0'..=b'9' => {
                    result += (digit & 0x0f) as f64 / div;
                    div *= 10_f64;
                }
                _ => return IntFloat::Err,
            }
        }
        if neg {
            IntFloat::Float(-result)
        } else {
            IntFloat::Float(result)
        }
    } else if neg {
        IntFloat::Int(-int_part)
    } else {
        IntFloat::Int(int_part)
    }
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
