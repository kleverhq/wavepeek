use crate::error::WavepeekError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FsdbTimeUnit {
    pub(super) factor: u64,
    pub(super) suffix: &'static str,
}

const UNITS_DESC: &[(&str, u128)] = &[
    ("s", 1_000_000_000_000_000_000_000),
    ("ms", 1_000_000_000_000_000_000),
    ("us", 1_000_000_000_000_000),
    ("ns", 1_000_000_000_000),
    ("ps", 1_000_000_000),
    ("fs", 1_000_000),
    ("as", 1_000),
    ("zs", 1),
];

pub(super) fn parse_scale_unit(raw: &str) -> Result<FsdbTimeUnit, WavepeekError> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(invalid_scale_unit(raw, "empty scale unit"));
    }
    if normalized.starts_with('-') {
        return Err(invalid_scale_unit(
            raw,
            "negative scale factors are not supported",
        ));
    }

    let split_at = normalized
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .ok_or_else(|| invalid_scale_unit(raw, "missing time unit suffix"))?;
    let (factor_text, suffix_text) = normalized.split_at(split_at);
    if factor_text.is_empty() {
        return Err(invalid_scale_unit(raw, "missing scale factor"));
    }
    if suffix_text.is_empty() {
        return Err(invalid_scale_unit(raw, "missing time unit suffix"));
    }
    if normalized[split_at..]
        .chars()
        .any(|ch| !(ch.is_ascii_alphabetic()))
    {
        return Err(invalid_scale_unit(raw, "malformed time unit suffix"));
    }

    let unit_zs = suffix_to_zeptoseconds(suffix_text)
        .ok_or_else(|| invalid_scale_unit(raw, "unsupported time unit suffix"))?;
    let factor_zs = parse_decimal_factor_to_zeptoseconds(factor_text, unit_zs)
        .map_err(|reason| invalid_scale_unit(raw, reason))?;
    if factor_zs == 0 {
        return Err(invalid_scale_unit(
            raw,
            "zero scale factors are not supported",
        ));
    }

    normalize_zeptoseconds_to_unit(factor_zs)
        .ok_or_else(|| invalid_scale_unit(raw, "normalized scale factor exceeds supported range"))
}

pub(super) fn normalize_time_unit(raw: &str) -> Result<String, WavepeekError> {
    let unit = parse_scale_unit(raw)?;
    Ok(format!("{}{}", unit.factor, unit.suffix))
}

pub(super) fn normalize_raw_time(raw: u64, unit: FsdbTimeUnit) -> Result<String, WavepeekError> {
    let scaled = raw.checked_mul(unit.factor).ok_or_else(|| {
        WavepeekError::File("time value overflow while normalizing FSDB timestamps".to_string())
    })?;
    Ok(format!("{}{}", scaled, unit.suffix))
}

fn suffix_to_zeptoseconds(suffix: &str) -> Option<u128> {
    match suffix {
        "zs" | "z" => Some(1),
        "as" | "a" => Some(1_000),
        "fs" | "f" => Some(1_000_000),
        "ps" | "p" => Some(1_000_000_000),
        "ns" | "n" => Some(1_000_000_000_000),
        "us" | "u" => Some(1_000_000_000_000_000),
        "ms" | "m" => Some(1_000_000_000_000_000_000),
        "s" => Some(1_000_000_000_000_000_000_000),
        _ => None,
    }
}

fn parse_decimal_factor_to_zeptoseconds(factor: &str, unit_zs: u128) -> Result<u128, &'static str> {
    let dot_count = factor.chars().filter(|ch| *ch == '.').count();
    if dot_count > 1 {
        return Err("malformed scale factor");
    }
    if factor == "." {
        return Err("malformed scale factor");
    }
    if !factor.chars().all(|ch| ch.is_ascii_digit() || ch == '.') {
        return Err("malformed scale factor");
    }

    let mut digits = String::with_capacity(factor.len());
    let mut fractional_digits = 0u32;
    let mut seen_dot = false;
    for ch in factor.chars() {
        if ch == '.' {
            seen_dot = true;
            continue;
        }
        digits.push(ch);
        if seen_dot {
            fractional_digits = fractional_digits
                .checked_add(1)
                .ok_or("scale factor has too many fractional digits")?;
        }
    }

    if digits.is_empty() || digits.chars().all(|ch| ch == '0') {
        return Ok(0);
    }

    let numerator = digits
        .parse::<u128>()
        .map_err(|_| "scale factor exceeds supported range")?;
    let denominator = 10u128
        .checked_pow(fractional_digits)
        .ok_or("scale factor has too many fractional digits")?;
    let scaled = numerator
        .checked_mul(unit_zs)
        .ok_or("scale factor exceeds supported range")?;
    if scaled % denominator != 0 {
        return Err("fractional scale unit cannot be represented exactly");
    }
    Ok(scaled / denominator)
}

fn normalize_zeptoseconds_to_unit(value_zs: u128) -> Option<FsdbTimeUnit> {
    for (suffix, unit_zs) in UNITS_DESC {
        if value_zs.is_multiple_of(*unit_zs) {
            let factor = value_zs / *unit_zs;
            let factor = u64::try_from(factor).ok()?;
            return Some(FsdbTimeUnit { factor, suffix });
        }
    }
    None
}

fn invalid_scale_unit(raw: &str, reason: &'static str) -> WavepeekError {
    WavepeekError::File(format!("unsupported FSDB scale unit '{raw}': {reason}"))
}

#[cfg(test)]
mod tests {
    use super::{FsdbTimeUnit, normalize_raw_time, normalize_time_unit, parse_scale_unit};

    #[test]
    fn fsdb_time_accepts_integer_units_and_short_suffixes() {
        for (raw, expected) in [
            ("1ns", "1ns"),
            ("10ps", "10ps"),
            ("100ps", "100ps"),
            ("1n", "1ns"),
            ("1p", "1ps"),
            ("1u", "1us"),
            ("1m", "1ms"),
            ("1s", "1s"),
            ("  1NS  ", "1ns"),
        ] {
            assert_eq!(normalize_time_unit(raw).expect(raw), expected);
        }
    }

    #[test]
    fn fsdb_time_accepts_exact_fractional_short_forms() {
        for (raw, expected) in [("0.1n", "100ps"), ("0.01n", "10ps"), ("0.001n", "1ps")] {
            assert_eq!(normalize_time_unit(raw).expect(raw), expected);
        }
    }

    #[test]
    fn fsdb_time_rejects_malformed_or_unsupported_units() {
        for raw in [
            "",
            "ns",
            "1",
            "-1ns",
            "0ns",
            "1xs",
            "1.2.3ns",
            ".ns",
            "0.0000000000001zs",
        ] {
            let error = normalize_time_unit(raw).expect_err(raw).to_string();
            assert!(
                error.contains("unsupported FSDB scale unit"),
                "unexpected error for {raw:?}: {error}"
            );
        }
    }

    #[test]
    fn fsdb_time_normalizes_raw_integer_tags() {
        let unit = parse_scale_unit("0.1n").expect("scale unit should parse");
        assert_eq!(
            unit,
            FsdbTimeUnit {
                factor: 100,
                suffix: "ps"
            }
        );
        assert_eq!(
            normalize_raw_time(42, unit).expect("time should normalize"),
            "4200ps"
        );
    }

    #[test]
    fn fsdb_time_rejects_raw_time_overflow() {
        let error = normalize_raw_time(
            u64::MAX,
            FsdbTimeUnit {
                factor: 2,
                suffix: "ns",
            },
        )
        .expect_err("overflow should fail")
        .to_string();
        assert!(error.contains("overflow"));
    }
}
