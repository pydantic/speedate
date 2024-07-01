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
    // we could eventually expose the number of decimal digits here, but it's
    // not needed for now
    let (float, _decimal_digits_count) = float_parse_bytes(s.as_bytes());
    float
}

/// Parse bytes as an float.
pub fn float_parse_bytes(s: &[u8]) -> (IntFloat, Option<usize>) {
    let (neg, first_digit, digits) = match s {
        [b'-', first, digits @ ..] => (true, first, digits),
        [b'+', first, digits @ ..] | [first, digits @ ..] => (false, first, digits),
        _ => return (IntFloat::Err, None),
    };

    let mut int_part = match first_digit {
        b'0' => 0,
        b'1'..=b'9' => (first_digit & 0x0f) as i64,
        _ => return (IntFloat::Err, None),
    };

    let mut found_dot = false;

    let mut bytes = digits.iter().copied();

    for digit in bytes.by_ref() {
        match digit {
            b'0'..=b'9' => {
                int_part = match int_part.checked_mul(10) {
                    Some(i) => i,
                    None => return (IntFloat::Err, None),
                };
                int_part = match int_part.checked_add((digit & 0x0f) as i64) {
                    Some(i) => i,
                    None => return (IntFloat::Err, None),
                };
            }
            b'.' => {
                found_dot = true;
                break;
            }
            _ => return (IntFloat::Err, None),
        }
    }

    if found_dot {
        let mut result = int_part as f64; // Integer part
        let mut frac_integers: u64 = 0; // To accumulate fractional part as integer
        let mut decimal_digits = 0; // Count of digits in the fractional part

        for digit in bytes {
            match digit {
                b'0'..=b'9' => {
                    decimal_digits += 1;
                    frac_integers = frac_integers * 10 + (digit & 0x0f) as u64;
                }
                _ => return (IntFloat::Err, Some(decimal_digits)),
            }
        }

        // Convert fractional part to f64 and divide by 10^decimal_digits
        let frac_as_f64 = frac_integers as f64 / 10_f64.powi(decimal_digits as i32);
        result += frac_as_f64;

        if neg {
            (IntFloat::Float(-result), Some(decimal_digits))
        } else {
            (IntFloat::Float(result), Some(decimal_digits))
        }
    } else if neg {
        (IntFloat::Int(-int_part), None)
    } else {
        (IntFloat::Int(int_part), None)
    }
}
