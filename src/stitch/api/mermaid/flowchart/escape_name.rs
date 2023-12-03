pub fn escape_name(name: &str) -> String {
    name.replace(' ', "_")
}
