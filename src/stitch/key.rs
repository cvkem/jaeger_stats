#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key {
    pub process: String,
    pub operation: String,
}

impl ToString for Key {
    fn to_string(&self) -> String {
        format!("{}/{}", self.process, self.operation)
    }
}
