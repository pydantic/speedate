/// macro for expected values
macro_rules! expect_ok_or_error {
    ($type:ty, $name:ident, ok, $input:literal, $expected:expr) => {
        paste::item! {
            #[test]
            fn [< expect_ $name _ok >]() {
                let v = <$type>::parse_str($input).unwrap();
                assert_eq!(v.to_string(), $expected);
            }
        }
    };
    ($type:ty, $name:ident, err, $input:literal, $error:expr) => {
        paste::item! {
            #[allow(non_snake_case)]
            #[test]
            fn [< expect_ $name _ $error _error >]() {
                match <$type>::parse_str($input) {
                    Ok(t) => panic!("unexpectedly valid: {:?} -> {:?}", $input, t),
                    Err(e) => assert_eq!(e, ParseError::$error),
                }
            }
        }
    };
}
pub(crate) use expect_ok_or_error;

/// macro to define many tests for expected values
macro_rules! param_tests {
    ($type:ty, $($name:ident: $ok_or_err:ident => $input:literal, $expected:expr;)*) => {
        $(
            common::expect_ok_or_error!($type, $name, $ok_or_err, $input, $expected);
        )*
    }
}
pub(crate) use param_tests;
