use chrono::Utc;

// Matches the format `2024-01-04T19:52:04.268331Z`
// This mimics the time codes sent by the current version of Jupyter Lab
pub fn iso_8601_Z_now() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso_8601_Z_now() {
        println!("{:?}", iso_8601_Z_now())
        
    }
}