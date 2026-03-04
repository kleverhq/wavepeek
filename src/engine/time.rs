use crate::error::WavepeekError;
use crate::waveform::WaveformMetadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParsedTime {
    pub(crate) value: u64,
    pub(crate) unit: TimeUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimeUnit {
    Zs,
    As,
    Fs,
    Ps,
    Ns,
    Us,
    Ms,
    S,
}

impl TimeUnit {
    fn suffix(self) -> &'static str {
        match self {
            Self::Zs => "zs",
            Self::As => "as",
            Self::Fs => "fs",
            Self::Ps => "ps",
            Self::Ns => "ns",
            Self::Us => "us",
            Self::Ms => "ms",
            Self::S => "s",
        }
    }

    fn multiplier_in_zeptoseconds(self) -> u128 {
        match self {
            Self::Zs => 1,
            Self::As => 1_000,
            Self::Fs => 1_000_000,
            Self::Ps => 1_000_000_000,
            Self::Ns => 1_000_000_000_000,
            Self::Us => 1_000_000_000_000_000,
            Self::Ms => 1_000_000_000_000_000_000,
            Self::S => 1_000_000_000_000_000_000_000,
        }
    }

    fn parse(token: &str) -> Option<Self> {
        match token {
            "zs" => Some(Self::Zs),
            "as" => Some(Self::As),
            "fs" => Some(Self::Fs),
            "ps" => Some(Self::Ps),
            "ns" => Some(Self::Ns),
            "us" => Some(Self::Us),
            "ms" => Some(Self::Ms),
            "s" => Some(Self::S),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DumpTimeContext {
    pub(crate) dump_tick: ParsedTime,
    pub(crate) dump_tick_zs: u128,
    pub(crate) dump_start_zs: u128,
    pub(crate) dump_end_zs: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimeValidationError {
    RequiresUnits,
    InvalidToken,
    TooLarge,
    OutOfBounds,
    NotAligned,
    RawOutOfRange,
}

pub(crate) fn parse_time_token(token: &str) -> Option<ParsedTime> {
    let split_at = token.find(|ch: char| !ch.is_ascii_digit())?;
    if split_at == 0 || split_at >= token.len() {
        return None;
    }

    let value = token[..split_at].parse::<u64>().ok()?;
    let unit = TimeUnit::parse(&token[split_at..])?;
    Some(ParsedTime { value, unit })
}

pub(crate) fn as_zeptoseconds(time: ParsedTime) -> Option<u128> {
    u128::from(time.value).checked_mul(time.unit.multiplier_in_zeptoseconds())
}

pub(crate) fn ensure_non_zero_dump_tick(dump_tick_zs: u128) -> Result<(), WavepeekError> {
    if dump_tick_zs == 0 {
        return Err(WavepeekError::Internal(
            "waveform metadata time_unit must be non-zero".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn format_raw_timestamp(
    raw_time: u64,
    time_unit: ParsedTime,
) -> Result<String, WavepeekError> {
    let normalized = raw_time.checked_mul(time_unit.value).ok_or_else(|| {
        WavepeekError::Internal("normalized time overflow while formatting timestamp".to_string())
    })?;
    Ok(format!("{normalized}{}", time_unit.unit.suffix()))
}

pub(crate) fn parse_dump_time_context(
    metadata: &WaveformMetadata,
) -> Result<DumpTimeContext, WavepeekError> {
    let dump_tick = parse_time_token(metadata.time_unit.as_str()).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "waveform metadata contains invalid time_unit '{}': expected <integer><unit>",
            metadata.time_unit
        ))
    })?;
    let dump_tick_zs = as_zeptoseconds(dump_tick).ok_or_else(|| {
        WavepeekError::Internal(
            "waveform metadata time_unit overflowed during conversion".to_string(),
        )
    })?;
    ensure_non_zero_dump_tick(dump_tick_zs)?;

    let dump_start = parse_time_token(metadata.time_start.as_str()).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "waveform metadata contains invalid time_start '{}': expected <integer><unit>",
            metadata.time_start
        ))
    })?;
    let dump_start_zs = as_zeptoseconds(dump_start).ok_or_else(|| {
        WavepeekError::Internal(
            "waveform metadata time_start overflowed during conversion".to_string(),
        )
    })?;

    let dump_end = parse_time_token(metadata.time_end.as_str()).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "waveform metadata contains invalid time_end '{}': expected <integer><unit>",
            metadata.time_end
        ))
    })?;
    let dump_end_zs = as_zeptoseconds(dump_end).ok_or_else(|| {
        WavepeekError::Internal(
            "waveform metadata time_end overflowed during conversion".to_string(),
        )
    })?;

    Ok(DumpTimeContext {
        dump_tick,
        dump_tick_zs,
        dump_start_zs,
        dump_end_zs,
    })
}

pub(crate) fn validate_time_token_to_raw(
    token: &str,
    context: DumpTimeContext,
    require_units: bool,
) -> Result<u64, TimeValidationError> {
    if require_units && token.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(TimeValidationError::RequiresUnits);
    }

    let parsed = parse_time_token(token).ok_or(TimeValidationError::InvalidToken)?;
    let parsed_zs = as_zeptoseconds(parsed).ok_or(TimeValidationError::TooLarge)?;

    if parsed_zs < context.dump_start_zs || parsed_zs > context.dump_end_zs {
        return Err(TimeValidationError::OutOfBounds);
    }

    if parsed_zs % context.dump_tick_zs != 0 {
        return Err(TimeValidationError::NotAligned);
    }

    let raw = parsed_zs / context.dump_tick_zs;
    u64::try_from(raw).map_err(|_| TimeValidationError::RawOutOfRange)
}

#[cfg(test)]
mod tests {
    use super::{
        DumpTimeContext, ParsedTime, TimeUnit, TimeValidationError, as_zeptoseconds,
        ensure_non_zero_dump_tick, format_raw_timestamp, parse_dump_time_context, parse_time_token,
        validate_time_token_to_raw,
    };
    use crate::waveform::WaveformMetadata;

    fn metadata() -> WaveformMetadata {
        WaveformMetadata {
            time_unit: "1ns".to_string(),
            time_start: "0ns".to_string(),
            time_end: "10ns".to_string(),
        }
    }

    #[test]
    fn parse_time_token_requires_integer_and_unit() {
        assert_eq!(
            parse_time_token("10ns"),
            Some(ParsedTime {
                value: 10,
                unit: TimeUnit::Ns
            })
        );
        assert_eq!(parse_time_token("100"), None);
        assert_eq!(parse_time_token("ns"), None);
        assert_eq!(parse_time_token("10NS"), None);
    }

    #[test]
    fn zeptoseconds_conversion_supports_cross_unit_comparison() {
        let one_ns = as_zeptoseconds(ParsedTime {
            value: 1,
            unit: TimeUnit::Ns,
        })
        .expect("1ns should convert");
        let thousand_ps = as_zeptoseconds(ParsedTime {
            value: 1000,
            unit: TimeUnit::Ps,
        })
        .expect("1000ps should convert");

        assert_eq!(one_ns, thousand_ps);
    }

    #[test]
    fn raw_timestamp_formatting_uses_dump_time_unit() {
        let formatted = format_raw_timestamp(
            10,
            ParsedTime {
                value: 1,
                unit: TimeUnit::Ns,
            },
        )
        .expect("formatting should succeed");
        assert_eq!(formatted, "10ns");

        let formatted = format_raw_timestamp(
            3,
            ParsedTime {
                value: 10,
                unit: TimeUnit::Ps,
            },
        )
        .expect("formatting should succeed");
        assert_eq!(formatted, "30ps");
    }

    #[test]
    fn dump_tick_must_be_non_zero() {
        let error = ensure_non_zero_dump_tick(0).expect_err("zero tick must fail");
        assert_eq!(
            error.to_string(),
            "error: internal: waveform metadata time_unit must be non-zero"
        );

        ensure_non_zero_dump_tick(1).expect("non-zero tick must pass");
    }

    #[test]
    fn parse_dump_time_context_extracts_dump_bounds_and_tick() {
        let context = parse_dump_time_context(&metadata()).expect("metadata should parse");

        assert_eq!(
            context.dump_tick,
            ParsedTime {
                value: 1,
                unit: TimeUnit::Ns,
            }
        );
        assert_eq!(
            context.dump_tick_zs, 1_000_000_000_000,
            "1ns should convert to zeptoseconds"
        );
        assert_eq!(context.dump_start_zs, 0);
        assert_eq!(context.dump_end_zs, 10_000_000_000_000);
    }

    #[test]
    fn validate_time_token_to_raw_rejects_requires_units_and_invalid_tokens() {
        let context = parse_dump_time_context(&metadata()).expect("metadata should parse");

        assert_eq!(
            validate_time_token_to_raw("10", context, true),
            Err(TimeValidationError::RequiresUnits)
        );
        assert_eq!(
            validate_time_token_to_raw("1.5ns", context, true),
            Err(TimeValidationError::InvalidToken)
        );
    }

    #[test]
    fn validate_time_token_to_raw_rejects_too_large_bounds_and_misalignment() {
        let context = parse_dump_time_context(&metadata()).expect("metadata should parse");
        let too_large_token = format!("{}s", u64::MAX);

        assert_eq!(
            validate_time_token_to_raw(too_large_token.as_str(), context, false),
            Err(TimeValidationError::TooLarge)
        );
        assert_eq!(
            validate_time_token_to_raw("11ns", context, false),
            Err(TimeValidationError::OutOfBounds)
        );
        assert_eq!(
            validate_time_token_to_raw("15ps", context, false),
            Err(TimeValidationError::NotAligned)
        );
    }

    #[test]
    fn validate_time_token_to_raw_rejects_raw_range_overflow() {
        let token = format!("{}ns", u64::MAX);
        let end_zs = as_zeptoseconds(ParsedTime {
            value: u64::MAX,
            unit: TimeUnit::Ns,
        })
        .expect("raw-overflow test token should convert to zeptoseconds");
        let context = DumpTimeContext {
            dump_tick: ParsedTime {
                value: 1,
                unit: TimeUnit::Zs,
            },
            dump_tick_zs: 1,
            dump_start_zs: 0,
            dump_end_zs: end_zs,
        };

        assert_eq!(
            validate_time_token_to_raw(token.as_str(), context, false),
            Err(TimeValidationError::RawOutOfRange)
        );
    }
}
