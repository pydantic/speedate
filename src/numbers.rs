/// Parse a string as an int.
///
/// This is around 2x faster than using `str::parse::<i64>()`
pub fn int_parse_str(s: &str) -> Option<i64> {
    fast_parse_int(s.bytes())
}

/// Parse bytes as an int.
pub fn int_parse_bytes(s: &[u8]) -> Option<i64> {
    fast_parse_int(s.iter().map(|&c| c))
}

pub fn fast_parse_int<I: Iterator>(mut bytes: I) -> Option<i64>
where
    I: Iterator<Item = u8>,
{
    let mut result: i64 = 0;
    let neg = match bytes.next() {
        Some(b'-') => true,
        Some(b'+') => false,
        Some(c) if (b'0'..=b'9').contains(&c) => {
            result = (c & 0x0f) as i64;
            false
        }
        _ => return None,
    };

    for digit in bytes {
        match digit {
            b'0'..=b'9' => {
                result = result.checked_mul(10)?;
                result = result.checked_add((digit & 0x0f) as i64)?
            }
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
    fast_parse_float(s.bytes())
}

/// Parse bytes as an float.
pub fn float_parse_bytes(s: &[u8]) -> IntFloat {
    fast_parse_float(s.iter().map(|&c| c))
}

pub fn fast_parse_float<I: Iterator>(mut bytes: I) -> IntFloat
where
    I: Iterator<Item = u8>,
{
    let mut int_part: i64 = 0;
    let neg = match bytes.next() {
        Some(b'-') => true,
        Some(c) if (b'0'..=b'9').contains(&c) => {
            int_part = (c & 0x0f) as i64;
            false
        }
        _ => return IntFloat::Err,
    };

    let mut found_dot = false;

    loop {
        let digit = match bytes.next() {
            Some(c) => c,
            None => break,
        };
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
