use std::num::ParseIntError;

pub fn parse_number(number_str: &str) -> Result<u64, ParseIntError> {
    if number_str.starts_with("0x") {
        u64::from_str_radix(&number_str[2..], 16)
    } else if number_str.starts_with("0") {
        u64::from_str_radix(number_str, 8)
    } else {
        u64::from_str_radix(number_str, 10)
    }
}

pub fn parse_number_or_zero(number_str: &str) -> u64 {
    match parse_number(number_str) {
        Ok(number) => number,
        _ => 0
    }
}
