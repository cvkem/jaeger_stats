use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

impl Default for Version {
    fn default() -> Self {
        Version { major: 0, minor: 2 }
    }
}

impl Version {

    pub fn new(major: u16, minor: u16) -> Self {
        Self{major, minor}
    }
}
