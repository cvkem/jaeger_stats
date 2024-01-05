const TO_REPLACE: [char; 5] = [' ', '{', '}', '(', ')'];
const REPLACE_CHAR: &str = "_";

/// escape a name such that it can be used as valid Mermaid label if the current input can not be used as label, otherwise None
pub fn escape_mermaid_label(name: &str) -> Option<String> {
    if name.contains(&TO_REPLACE) {
        Some(name.replace(&TO_REPLACE, REPLACE_CHAR))
    } else {
        None
    }
}
