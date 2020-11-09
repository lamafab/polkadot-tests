use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task<T> {
    name: Vec<T>
}

struct VarPool {
    pool: HashMap<(), ()>,
}

impl VarPool {
    fn new() -> Self {
        VarPool {
            pool: HashMap::new(),
        }
    }
}

struct TaskList {
    tasks: Vec<()>,
}

impl TaskList {
    fn new() -> Self {
        TaskList {
            tasks: vec![],
        }
    }
}
