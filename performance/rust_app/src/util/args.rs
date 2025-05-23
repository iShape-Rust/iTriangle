use std::collections::HashMap;
use std::env;

pub(crate) struct EnvArgs {
    map: HashMap<String, String>,
}
#[allow(dead_code)]
impl EnvArgs {
    pub(crate) fn new() -> Self {
        let args = env::args();
        let mut args_iter = args.peekable();
        let mut args_map = HashMap::new();

        while let Some(arg) = args_iter.next() {
            if arg.starts_with("--") {
                let key = arg.trim_start_matches("--").to_owned();
                // If the next argument is also a key, store a boolean flag; otherwise, store the value.
                let value = if args_iter.peek().is_some_and(|a| a.starts_with("--")) {
                    "true".to_string()
                } else {
                    args_iter.next().unwrap()
                };
                args_map.insert(key, value);
            }
        }

        Self { map: args_map }
    }

    pub(crate) fn get_usize(&self, name: &str) -> usize {
        let value = self.map.get(name).unwrap_or_else(|| panic!("{} is not set", name));
        value
            .parse()
            .unwrap_or_else(|_| panic!("Unable to {} as an usize", name))
    }

    pub(crate) fn get_bool(&self, name: &str) -> bool {
        let value = self.map.get(name).unwrap_or_else(|| panic!("{} is not set", name));
        value
            .parse()
            .unwrap_or_else(|_| panic!("Unable to {} as a boolean", name))
    }

    pub(crate) fn set_bool(&mut self, name: &str, value: bool) {
        self.map.insert(name.to_string(), value.to_string());
    }

    pub(crate) fn set_usize(&mut self, name: &str, value: usize) {
        self.map.insert(name.to_string(), value.to_string());
    }
}
