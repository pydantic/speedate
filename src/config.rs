use crate::{ConfigError, MicrosecondsPrecisionOverflowBehavior};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TimestampUnit {
    /// Interpret as seconds since the UNIX epoch.
    Second,
    /// Interpret as milliseconds since the UNIX epoch.
    Millisecond,
    /// Let the parser infer units based on value length.
    #[default]
    Infer,
}

impl TryFrom<&str> for TimestampUnit {
    type Error = ConfigError;
    fn try_from(value: &str) -> Result<Self, ConfigError> {
        match value.to_lowercase().as_str() {
            "s" => Ok(Self::Second),
            "ms" => Ok(Self::Millisecond),
            "infer" => Ok(Self::Infer),
            _ => Err(ConfigError::UnknownTimestampUnitString),
        }
    }
}

/// Configuration for parsing `Date`.
#[derive(Debug, Clone)]
pub struct DateConfig {
    /// How to interpret numeric timestamps (seconds, milliseconds, etc.).
    pub timestamp_unit: TimestampUnit,
}

impl Default for DateConfig {
    fn default() -> Self {
        DateConfig {
            timestamp_unit: TimestampUnit::Infer,
        }
    }
}

/// Configuration for parsing `DateTime`.
#[derive(Debug, Clone)]
pub struct DateTimeConfig {
    /// How to interpret numeric timestamps (seconds, milliseconds, etc.).
    pub timestamp_unit: TimestampUnit,
    /// Configuration used when parsing the time component.
    pub time_config: TimeConfig,
}

impl Default for DateTimeConfig {
    fn default() -> Self {
        DateTimeConfig {
            timestamp_unit: TimestampUnit::Infer,
            time_config: TimeConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TimeConfig {
    pub microseconds_precision_overflow_behavior: MicrosecondsPrecisionOverflowBehavior,
    pub unix_timestamp_offset: Option<i32>,
}

impl TimeConfig {
    pub fn builder() -> TimeConfigBuilder {
        TimeConfigBuilder::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TimeConfigBuilder {
    microseconds_precision_overflow_behavior: Option<MicrosecondsPrecisionOverflowBehavior>,
    unix_timestamp_offset: Option<i32>,
}

impl TimeConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn microseconds_precision_overflow_behavior(
        mut self,
        microseconds_precision_overflow_behavior: MicrosecondsPrecisionOverflowBehavior,
    ) -> Self {
        self.microseconds_precision_overflow_behavior = Some(microseconds_precision_overflow_behavior);
        self
    }
    pub fn unix_timestamp_offset(mut self, unix_timestamp_offset: Option<i32>) -> Self {
        self.unix_timestamp_offset = unix_timestamp_offset;
        self
    }
    pub fn build(self) -> TimeConfig {
        TimeConfig {
            microseconds_precision_overflow_behavior: self.microseconds_precision_overflow_behavior.unwrap_or_default(),
            unix_timestamp_offset: self.unix_timestamp_offset,
        }
    }
}
