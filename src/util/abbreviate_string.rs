pub fn abbreviate_string(s: &str) -> String {
    let threshold = 10;
    let ellipsis = "...";

    if s.len() > threshold {
        let start = &s[..5]; // First 5 characters
        let end = &s[s.len()-5..]; // Last 5 characters
        format!("{}{}{}", start, ellipsis, end)
    } else {
        s.to_string()
    }
}