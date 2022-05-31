# speedate

[![CI](https://github.com/samuelcolvin/speedate/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/samuelcolvin/speedate/actions/workflows/ci.yml?query=branch%3Amain)
[![Coverage](https://codecov.io/gh/samuelcolvin/speedate/branch/main/graph/badge.svg?token=xCXg5aV9wM)](https://codecov.io/gh/samuelcolvin/speedate)

Fast and simple datetime, date, time and duration parsing for rust.

**speedate** is a lax† **RFC 3339** date and time parser, in other words, it parses common **ISO 8601**
formats.

**†** - all relaxations of from [RFC 3339](https://tools.ietf.org/html/rfc3339)
are compliant with [ISO 8601](https://en.wikipedia.org/wiki/ISO_8601).

The following formats are supported:
* Date: `YYYY-MM-DD`
* Time: `HH:MM:SS`
* Time: `HH:MM:SS.FFFFFF` 1 to 6 digits are allowed
* Time: `HH:MM`
* Date time: `YYYY-MM-DDTHH:MM:SS` - all the above time formats are allowed for the time part
* Date time: `YYYY-MM-DD HH:MM:SS` - `T`, `t`, ` ` and `_` are allowed as separators
* Date time: `YYYY-MM-DDTHH:MM:SSZ` - `Z` or `z` is allowed as timezone
* Date time: `YYYY-MM-DDTHH:MM:SS+08:00`- positive and negative timezone are allowed, as per ISO 8601, U+2212 minus `−`
  is allowed as well as ascii minus `-` (U+002D)
* Date time: `YYYY-MM-DDTHH:MM:SS+0800` - the colon (`:`) in the timezone is optional
* Duration: `PnYnMnDTnHnMnS` - ISO 8601 duration format,
  see [wikipedia](https://en.wikipedia.org/wiki/ISO_8601#Durations) for more details, `W` for weeks is also allowed
* Duration: `HH:MM:SS` - any of the above time formats are allowed to represent a duration
* Duration: `D days, HH:MM:SS` - time prefixed by `X days`, case-insensitive, spaces `s` and `,` are all optional
* Duration: `D d, HH:MM:SS` - time prefixed by `X d`, case-insensitive, spaces and `,` are optional
* Duration: `±...` - all duration formats shown here can be prefixed with `+` or `-` to indicate
  positive and negative durations respectively

This will be the datetime parsing logic for [pydantic-core](https://github.com/samuelcolvin/pydantic-core).

## Usage

```rust
use speedate::{DateTime, Date, Time};

fn main() {
    let dt = DateTime::parse_str("2022-01-01T12:13:14Z").unwrap();
    assert_eq!(
        dt,
        DateTime {
            date: Date {
                year: 2022,
                month: 1,
                day: 1,
            },
            time: Time {
                hour: 12,
                minute: 13,
                second: 14,
                microsecond: 0,
            },
            offset: Some(0),
        }
    );
    assert_eq!(dt.to_string(), "2022-01-01T12:13:14Z");
}
```

## Performance

**speedate** is significantly faster than
[chrono's `parse_from_rfc3339`](https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.parse_from_rfc3339)
and [iso8601](https://crates.io/crates/iso8601).

Micro-benchmarking from [`benches/main.rs`](https://github.com/samuelcolvin/speedate/blob/main/benches/main.rs):

```text
test compare_dt_error_chrono   ... bench:         192 ns/iter (+/- 0)
test compare_dt_error_iso8601  ... bench:         462 ns/iter (+/- 4)
test compare_dt_error_speedate ... bench:          26 ns/iter (+/- 0)
test compare_dt_ok_chrono      ... bench:         250 ns/iter (+/- 1)
test compare_dt_ok_iso8601     ... bench:         323 ns/iter (+/- 1)
test compare_dt_ok_speedate    ... bench:          36 ns/iter (+/- 0)
```

## Why not full iso8601?

ISO8601 has lots of allowed formats, see
[ijmacd.github.io/rfc3339-iso8601](https://ijmacd.github.io/rfc3339-iso8601/).

Most of these are unknown to most users, and not desired. This library aims to support the most common formats
without introducing ambiguity.
