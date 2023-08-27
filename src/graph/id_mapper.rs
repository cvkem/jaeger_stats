use std::{collections::HashMap, sync::Mutex};

/// Generates a series of ID with the prefix followed by an underscore and number, so prefix proc will return 'proc_0', 'proc_1'
pub struct IdGenerator {
    prefix: String,
    nr: Mutex<u64>,
}

impl IdGenerator {
    pub fn new(prefix: String) -> Self {
        let nr = Mutex::new(0);
        Self { prefix, nr }
    }

    pub fn get_id(&mut self) -> String {
        let mut guard = self.nr.lock().unwrap();
        let nr = *guard;
        *guard += 1;
        format!("{}_{nr}", self.prefix)
    }
}

/// IdMapper maps strings to clean id's that can be used safely in the dot-language.
pub struct IdMapper {
    id_generator: IdGenerator,
    mapping: HashMap<String, String>,
}

impl IdMapper {
    pub fn new(prefix: String) -> Self {
        let id_generator = IdGenerator::new(prefix);
        let mapping = HashMap::new();
        Self {
            id_generator,
            mapping,
        }
    }

    // TODO: check this error on the internet
    // pub fn map_key(&mut self, key: &str) -> &str {
    //   match self.mapping.get(key) {
    //     Some(id) => id,
    //     None => {
    //         let id = self.id_generator.get_id();
    //         // next line triggers error:
    //         // error[E0502]: cannot borrow `self.mapping` as mutable because it is also borrowed as immutable
    //         // which ssems incorrect as the mapping.get has finished
    //         // However, get key returns a reference which triggers this behavior as the compiler might conservaly suspect
    //         // this reference could point/refer to a value inside the object, while it actually is a normal value.
    //         self.mapping.insert(key.to_owned(), id);
    //         self.mapping.get(key).expect("Inserted key does not exist!!")
    //     }
    //}

    pub fn map_key(&mut self, key: &str) -> &str {
        self.mapping
            .entry(key.to_owned())
            .or_insert_with(|| self.id_generator.get_id())
    }
}
