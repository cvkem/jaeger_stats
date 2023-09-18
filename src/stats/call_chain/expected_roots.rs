use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExpectRoot {
    pub proc_oper: String,
    pub count: usize,
}

impl ExpectRoot {
    pub fn new(po: &str) -> Self {
        Self {
            proc_oper: po.to_owned(),
            count: 1,
        }
    }
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExpectedRoots(pub Vec<ExpectRoot>);

impl ExpectedRoots {
    /// update the count of the root (for a non-rooted trace)
    pub fn add_root(&mut self, root_proc_oper: &str) {
        let mut idx = None;
        for i in 0..(self.0.len()) {
            if self.0[i].proc_oper == root_proc_oper {
                idx = Some(i);
                break;
            }
        }
        match idx {
            Some(idx) => self.0[idx].count += 1,
            None => self.0.push(ExpectRoot::new(root_proc_oper)),
        }
    }

    pub fn get_frequent_end_point(&mut self) -> Option<String> {
        match self.0.len() {
            0 => None,
            1 => Some(self.0[0].proc_oper.to_string()),
            n => {
                println!("Select best key out of {n} end_points");
                let mut idx = 0;
                for i in 1..(self.0.len()) {
                    if self.0[i].count > self.0[idx].count {
                        idx = i;
                    }
                }
                Some(self.0[idx].proc_oper.to_string())
            }
        }
    }
}
