use serde::Serialize;

use crate::cli::at::AtArgs;
use crate::engine::time::{
    TimeValidationError, format_raw_timestamp, parse_dump_time_context, validate_time_token_to_raw,
};
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::waveform::{Waveform, WaveformMetadata};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AtSignalValue {
    #[serde(skip_serializing)]
    pub display: String,
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
    display: String,
    path: String,
}

pub fn run(args: AtArgs) -> Result<CommandResult, WavepeekError> {
    let mut waveform = Waveform::open(args.waves.as_path())?;
    let metadata = waveform.metadata()?;

    let requested_signals = resolve_requested_signals(&waveform, args.scope.as_deref(), &args)?;

    let dump_time = parse_dump_time_context(&metadata)?;
    let query_time_raw = validate_time_token_to_raw(args.time.as_str(), dump_time, false)
        .map_err(|error| map_at_time_validation_error(args.time.as_str(), &metadata, error))?;

    let canonical_paths = requested_signals
        .iter()
        .map(|signal| signal.path.clone())
        .collect::<Vec<_>>();
    let sampled = waveform.sample_signals_at_time(&canonical_paths, query_time_raw)?;

    let signals = requested_signals
        .into_iter()
        .zip(sampled)
        .map(|(requested, sampled)| AtSignalValue {
            display: requested.display,
            path: sampled.path,
            value: format_verilog_literal(sampled.width, sampled.bits.as_str()),
        })
        .collect::<Vec<_>>();

    let normalized_time = format_raw_timestamp(query_time_raw, dump_time.dump_tick)?;

    Ok(CommandResult {
        command: CommandName::At,
        json: args.json,
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: false,
            signals_abs: args.abs,
        },
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
        let display = token.trim();
        if display.is_empty() {
            return Err(WavepeekError::Args(
                "signal names must not be empty. See 'wavepeek at --help'.".to_string(),
            ));
        }

        let path = match scope {
            Some(scope) => format!("{scope}.{display}"),
            None => display.to_string(),
        };
        resolved.push(RequestedSignal {
            display: display.to_string(),
            path,
        });
    }

    Ok(resolved)
}

fn map_at_time_validation_error(
    token: &str,
    metadata: &WaveformMetadata,
    error: TimeValidationError,
) -> WavepeekError {
    match error {
        TimeValidationError::RequiresUnits | TimeValidationError::InvalidToken => {
            WavepeekError::Args(format!(
                "invalid time token '{token}': expected <integer><unit> (for example 10ns). See 'wavepeek at --help'."
            ))
        }
        TimeValidationError::TooLarge => WavepeekError::Args(format!(
            "time '{token}' is too large to process safely. See 'wavepeek at --help'."
        )),
        TimeValidationError::OutOfBounds => WavepeekError::Args(format!(
            "time '{token}' is outside dump bounds [{}, {}]. See 'wavepeek at --help'.",
            metadata.time_start, metadata.time_end
        )),
        TimeValidationError::NotAligned => WavepeekError::Args(format!(
            "time '{token}' is not aligned to dump resolution '{}'. See 'wavepeek at --help'.",
            metadata.time_unit
        )),
        TimeValidationError::RawOutOfRange => WavepeekError::Args(format!(
            "time '{token}' exceeds supported raw timestamp range. See 'wavepeek at --help'."
        )),
    }
}

pub(crate) fn format_verilog_literal(width: u32, bits: &str) -> String {
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
    use super::{bits_chunk_to_hex_digit, format_verilog_literal};

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
