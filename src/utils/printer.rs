pub fn pretty_print<T: serde::Serialize>(data: &T) -> String {
    let separator = "-".repeat(50);
    let formatted = match serde_json::to_string_pretty(data) {
        Ok(formatted) => formatted,
        Err(_) => String::from("Error formatting data"),
    };

    format!("{}\n{}\n{}", separator, formatted, separator)
}

// Optional: Add a version without separators
pub fn pretty_print_simple<T: serde::Serialize>(data: &T) -> String {
    serde_json::to_string_pretty(data).unwrap_or_else(|_| String::from("Error formatting data"))
}
