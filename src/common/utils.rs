pub fn get_key() -> [u8; 16] {
    u128::from_str_radix(
        option_env!("KEY").unwrap_or("A0B1C2D3E4F5061728394A5B6C7D8E9F"),
        16,
    )
    .unwrap()
    .to_le_bytes()
}

pub fn copy_into_array(source: &[u8], target: &mut [u8]) {
    let mut cntr = 0;

    for byte in source {
        target[cntr] = *byte;
        cntr += 1;
    }
}
