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
        assert_eq!(format_verilog_literal(0, ""), "0'h0");
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
