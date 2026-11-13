/// WavePeek's four-state interpretation of reader-provided logic states.
pub(crate) fn four_state(bit: char) -> char {
    match bit.to_ascii_lowercase() {
        '0' | 'l' => '0',
        '1' | 'h' => '1',
        'z' => 'z',
        _ => 'x',
    }
}
