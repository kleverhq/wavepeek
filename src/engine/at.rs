use serde::Serialize;

use crate::cli::at::AtArgs;
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::waveform::Waveform;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AtSignalValue {
    pub name: String,
    pub path: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AtData {
    pub time: String,
    pub signals: Vec<AtSignalValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestedSignal {
    name: String,
    path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedTime {
    value: u64,
    unit: TimeUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeUnit {
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

pub fn run(args: AtArgs) -> Result<CommandResult, WavepeekError> {
    let mut waveform = Waveform::open(args.waves.as_path())?;
    let metadata = waveform.metadata()?;

    let requested_signals = resolve_requested_signals(&waveform, args.scope.as_deref(), &args)?;

    let query_time = parse_time_token(args.time.as_str()).ok_or_else(|| {
        WavepeekError::Args(format!(
            "invalid time token '{}': expected <integer><unit> (for example 10ns). See 'wavepeek at --help'.",
            args.time
        ))
    })?;
    let query_time_zs = as_zeptoseconds(query_time).ok_or_else(|| {
        WavepeekError::Args(format!(
            "time '{}' is too large to process safely. See 'wavepeek at --help'.",
            args.time
        ))
    })?;

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

    let time_start = parse_time_token(metadata.time_start.as_str()).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "waveform metadata contains invalid time_start '{}': expected <integer><unit>",
            metadata.time_start
        ))
    })?;
    let time_start_zs = as_zeptoseconds(time_start).ok_or_else(|| {
        WavepeekError::Internal(
            "waveform metadata time_start overflowed during conversion".to_string(),
        )
    })?;

    let time_end = parse_time_token(metadata.time_end.as_str()).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "waveform metadata contains invalid time_end '{}': expected <integer><unit>",
            metadata.time_end
        ))
    })?;
    let time_end_zs = as_zeptoseconds(time_end).ok_or_else(|| {
        WavepeekError::Internal(
            "waveform metadata time_end overflowed during conversion".to_string(),
        )
    })?;

    if query_time_zs < time_start_zs || query_time_zs > time_end_zs {
        return Err(WavepeekError::Args(format!(
            "time '{}' is outside dump bounds [{}, {}]. See 'wavepeek at --help'.",
            args.time, metadata.time_start, metadata.time_end
        )));
    }

    if query_time_zs % dump_tick_zs != 0 {
        return Err(WavepeekError::Args(format!(
            "time '{}' is not aligned to dump resolution '{}'. See 'wavepeek at --help'.",
            args.time, metadata.time_unit
        )));
    }

    let query_time_raw_u128 = query_time_zs / dump_tick_zs;
    let query_time_raw = u64::try_from(query_time_raw_u128).map_err(|_| {
        WavepeekError::Args(format!(
            "time '{}' exceeds supported raw timestamp range. See 'wavepeek at --help'.",
            args.time
        ))
    })?;

    let canonical_paths = requested_signals
        .iter()
        .map(|signal| signal.path.clone())
        .collect::<Vec<_>>();
    let sampled = waveform.sample_signals_at_time(&canonical_paths, query_time_raw)?;

    let signals = requested_signals
        .into_iter()
        .zip(sampled)
        .map(|(requested, sampled)| AtSignalValue {
            name: requested.name,
            path: sampled.path,
            value: format_verilog_literal(sampled.width, sampled.bits.as_str()),
        })
        .collect::<Vec<_>>();

    let normalized_time = format_raw_timestamp(query_time_raw, dump_tick)?;

    Ok(CommandResult {
        command: CommandName::At,
        json: args.json,
        human_options: crate::engine::HumanRenderOptions::default(),
        data: CommandData::At(AtData {
            time: normalized_time,
            signals,
        }),
        warnings: Vec::new(),
    })
}

fn resolve_requested_signals(
    waveform: &Waveform,
    scope: Option<&str>,
    args: &AtArgs,
) -> Result<Vec<RequestedSignal>, WavepeekError> {
    if let Some(scope) = scope {
        waveform.signals_in_scope(scope)?;
    }

    let mut resolved = Vec::with_capacity(args.signals.len());
    for token in &args.signals {
        let name = token.trim();
        if name.is_empty() {
            return Err(WavepeekError::Args(
                "signal names must not be empty. See 'wavepeek at --help'.".to_string(),
            ));
        }

        let path = match scope {
            Some(scope) => format!("{scope}.{name}"),
            None => name.to_string(),
        };
        resolved.push(RequestedSignal {
            name: name.to_string(),
            path,
        });
    }

    Ok(resolved)
}

fn parse_time_token(token: &str) -> Option<ParsedTime> {
    let split_at = token.find(|ch: char| !ch.is_ascii_digit())?;
    if split_at == 0 || split_at >= token.len() {
        return None;
    }

    let value = token[..split_at].parse::<u64>().ok()?;
    let unit = TimeUnit::parse(&token[split_at..])?;
    Some(ParsedTime { value, unit })
}

fn as_zeptoseconds(time: ParsedTime) -> Option<u128> {
    u128::from(time.value).checked_mul(time.unit.multiplier_in_zeptoseconds())
}

fn format_raw_timestamp(raw_time: u64, time_unit: ParsedTime) -> Result<String, WavepeekError> {
    let normalized = raw_time.checked_mul(time_unit.value).ok_or_else(|| {
        WavepeekError::Internal("normalized time overflow while formatting timestamp".to_string())
    })?;
    Ok(format!("{normalized}{}", time_unit.unit.suffix()))
}

fn format_verilog_literal(width: u32, bits: &str) -> String {
    if width == 0 {
        return "0'h0".to_string();
    }

    let mut digits = String::with_capacity(bits.len().div_ceil(4));
    let first_group_len = {
        let rem = bits.len() % 4;
        if rem == 0 { 4 } else { rem }
    };

    let mut index = 0usize;
    while index < bits.len() {
        let chunk_len = if index == 0 { first_group_len } else { 4 };
        let chunk = &bits[index..(index + chunk_len)];
        digits.push(bits_chunk_to_hex_digit(chunk));
        index += chunk_len;
    }

    format!("{width}'h{digits}")
}

fn bits_chunk_to_hex_digit(chunk: &str) -> char {
    if chunk.chars().all(|ch| ch == 'z') {
        return 'z';
    }
    if chunk.chars().all(|ch| ch == '0' || ch == '1') {
        let mut value = 0u8;
        for ch in chunk.chars() {
            value = (value << 1)
                + match ch {
                    '0' => 0,
                    '1' => 1,
                    _ => unreachable!("binary chunk must contain only 0/1"),
                };
        }
        return char::from_digit(u32::from(value), 16).unwrap_or('x');
    }

    'x'
}

#[cfg(test)]
mod tests {
    use super::{
        ParsedTime, TimeUnit, as_zeptoseconds, bits_chunk_to_hex_digit, format_raw_timestamp,
        format_verilog_literal, parse_time_token,
    };

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
    fn verilog_literal_formatter_emits_lowercase_hex_and_unknowns() {
        assert_eq!(format_verilog_literal(8, "00001111"), "8'h0f");
        assert_eq!(format_verilog_literal(1, "1"), "1'h1");
        assert_eq!(format_verilog_literal(4, "zzzz"), "4'hz");
        assert_eq!(format_verilog_literal(4, "10xz"), "4'hx");
    }

    #[test]
    fn nibble_conversion_prefers_binary_then_z_then_x() {
        assert_eq!(bits_chunk_to_hex_digit("1010"), 'a');
        assert_eq!(bits_chunk_to_hex_digit("zz"), 'z');
        assert_eq!(bits_chunk_to_hex_digit("z1"), 'x');
        assert_eq!(bits_chunk_to_hex_digit("h"), 'x');
    }
}
