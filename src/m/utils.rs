

pub fn str_to_u32(str:&Option<String>) -> u32 {
  u32::from_str_radix(&str.clone().unwrap_or(String::from("0")).trim_start_matches("0x"), 16).unwrap()
}
pub fn validate_str_to_u32(str:&str) -> bool {
  u32::from_str_radix(str.trim_start_matches("0x"), 16).is_ok()
}

pub fn available_parallelism() -> usize {
  match std::thread::available_parallelism() {
    Ok(val) => val.get(),
    Err(_) => 1,
  }
}