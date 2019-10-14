pub fn remove_color<T: AsRef<str>>(text: T) -> String {
  let mut found_ampersand = false;

  text
    .as_ref()
    .chars()
    .filter(|&c| {
      if c == '&' {
        // we remove all amps but they're kept in chat if repeated
        found_ampersand = true;
        false
      } else if found_ampersand {
        found_ampersand = false;
        false
      } else {
        true
      }
    })
    .collect()
}
