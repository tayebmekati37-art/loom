use crate::ir::Function;
use std::collections::HashMap;

pub struct StranglerFig {
    legacy_functions: HashMap<String, Function>,
    modern_functions: HashMap<String, Function>,
    routing_table: HashMap<String, Routing>,
}

#[derive(Clone)]
pub enum Routing {
    Legacy,
    Modern,
    Mixed,
}

impl StranglerFig {
    pub fn new() -> Self {
        StranglerFig {
            legacy_functions: HashMap::new(),
            modern_functions: HashMap::new(),
            routing_table: HashMap::new(),
        }
    }

    pub fn add_legacy(&mut self, name: String, func: Function) {
        self.legacy_functions.insert(name.clone(), func);
        self.routing_table.insert(name, Routing::Legacy);
    }

    pub fn add_modern(&mut self, name: String, func: Function) {
        self.modern_functions.insert(name.clone(), func);
        self.routing_table.entry(name).or_insert(Routing::Modern);
    }

    pub fn set_routing(&mut self, name: &str, routing: Routing) {
        self.routing_table.insert(name.to_string(), routing);
    }

    pub fn generate_wrapper_code(&self, target_lang: &str) -> String {
        match target_lang {
            "python" => self.generate_python_wrapper(),
            _ => String::from("# Unsupported language"),
        }
    }

    fn generate_python_wrapper(&self) -> String {
        let mut code = String::new();
        for (name, routing) in &self.routing_table {
            match routing {
                Routing::Legacy => {
                    code.push_str(&format!(
                        "def {}(*args):\n    # Call legacy (not implemented)\n    pass\n\n",
                        name
                    ));
                }
                Routing::Modern => {
                    // Assume modern function exists with same name
                    code.push_str(&format!(
                        "def {}(*args):\n    return modern_{}(*args)\n\n",
                        name, name
                    ));
                }
                Routing::Mixed => {
                    code.push_str(&format!(
                        "def {}(*args):\n    # Route based on feature flag\n    if is_feature_enabled('{}'):\n        return modern_{}(*args)\n    else:\n        # Call legacy (not implemented)\n        pass\n\n",
                        name, name, name
                    ));
                }
            }
        }
        code
    }
}