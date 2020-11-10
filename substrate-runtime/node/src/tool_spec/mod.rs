use crate::Result;
use std::collections::HashMap;
use std::fs;

mod block;

pub struct ToolSpec {
    var_pool: VarPool,
    task_list: TaskList,
}

impl ToolSpec {
    pub fn new_from_file(path: &str) -> Result<Self> {
        let yaml_file = fs::read_to_string(path)?;
        let yaml_blocks: Vec<YamlItem> = serde_yaml::from_str(&yaml_file)?;

        let mut var_pool = VarPool::new();
        let mut task_list = TaskList::new();

        for item in yaml_blocks {
            match item {
                YamlItem::Task(task) => {}
                YamlItem::Vars(vars) => {}
            }
        }

        Ok(ToolSpec {
            var_pool: var_pool,
            task_list: task_list,
        })
    }
    pub fn run(self) {}
}

struct TaskOutcome<T> {
    name: String,
    out: Box<T>,
}

fn task_parser(properties: HashMap<KeyType, ValType>) -> Result<()> {
    let mut task = None;
    let mut local_var_pool = VarPool::new();
    let mut register = false;

    for (key, _) in &properties {
        match key {
            KeyType::TaskType(task_ty) => {
                if task.is_none() {
                    task = Some(task_ty)
                } else {
                    return Err(failure::err_msg("Only one task per yaml block is allowed"));
                }
            }
            KeyType::Keyword(keyword) => match keyword {
                Keyword::Register => unimplemented!(),
                Keyword::Loop => register = true,
                Keyword::Vars => unimplemented!(),
            },
        }
    }

    let task = task.ok_or(failure::err_msg("Not task specified"));

    for (_, val) in &properties {}

    /*
    match task_ty {
        TaskType::Block => unimplemented!(),
        TaskType::PalletBalances => unimplemented!(),
        TaskType::Execute => unimplemented!(),
    }
    */

    Ok(())
}

struct PrimitiveConverter<'a> {
    global_var_pool: &'a VarPool,
    local_var_pool: &'a VarPool,
}

impl<'a> PrimitiveConverter<'a> {
    fn new(global: &'a VarPool, local: &'a VarPool) -> Self {
        PrimitiveConverter {
            global_var_pool: global,
            local_var_pool: local,
        }
    }
    fn process_properties(&self, properties: &mut HashMap<String, ValType>) -> Result<()> {
        for (_, val) in properties {
            self.process_value_ty(val)?;
        }

        Ok(())
    }
    fn process_value_ty(&self, val_ty: &mut ValType) -> Result<()> {
        match val_ty {
            ValType::List(v) => unimplemented!(),
            ValType::Map(map) => {
                for (_, v) in map {
                    self.process_value_ty(v)?;
                }
            }
            ValType::SingleValue(val) => self.process_yaml_value(val)?,
            ValType::Variable(name) => {
                if let Some(v) = self.local_var_pool.get(name) {
                    *val_ty = ValType::SingleValue(v.clone())
                } else if let Some(v) = self.global_var_pool.get(name) {
                    *val_ty = ValType::SingleValue(v.clone())
                } else {
                    return Err(failure::err_msg(""))
                }
            }
            ValType::LoopVariable(v) => unimplemented!(),
        }

        Ok(())
    }
    fn process_yaml_value(&self, value: &mut serde_yaml::Value) -> Result<()> {
        if let Some(val) = value.as_str() {
            if val.contains("{{") && val.contains("}}") {
                let val = val.replace("{{", "").replace("}}", "");
                let trimmed = val.trim();

                if trimmed.starts_with("item.") {
                    //ValueType::LoopVariable(trimmed.replace("item.", "").to_string())
                    unimplemented!()
                } else {
                    let var_name = VariableName(trimmed.to_string());
                    if let Some(var) = self.local_var_pool.get(&var_name) {
                        *value = var.clone();
                    } else if let Some(var) = self.global_var_pool.get(&var_name) {
                        *value = var.clone();
                    } else {
                        return Err(failure::err_msg(format!("Variable \"{}\" not found", var_name.0)))
                    }
                }
            }
        } else if let Some(seq) = value.as_sequence_mut() {
            for val in seq {
                self.process_yaml_value(val)?;
            }
        } else if let Some(map) = value.as_mapping_mut() {
            for (_, val) in map {
                self.process_yaml_value(val)?;
            }
        }

        Ok(())
    }
}

#[test]
fn tool_spec_init() {
    ToolSpec::new_from_file("../examples/block_builder.yml").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum YamlItem {
    Task(Task),
    Vars(Vars),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Vars {
    vars: VarsType,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum VarsType {
    Map(HashMap<VariableName, ValType>),
    ListMap(Vec<HashMap<VariableName, ValType>>),
    List(Vec<ValType>),
    SingleValue(ValType),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum ValType {
    List(Vec<ValType>),
    Map(HashMap<String, ValType>),
    SingleValue(serde_yaml::Value),
    Variable(VariableName),
    LoopVariable(VariableName),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct VariableName(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    name: String,
    #[serde(flatten)]
    properties: HashMap<KeyType, ValType>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
enum KeyType {
    TaskType(TaskType),
    Keyword(Keyword),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum Keyword {
    #[serde(rename = "register")]
    Register,
    #[serde(rename = "loop")]
    Loop,
    #[serde(rename = "vars")]
    Vars,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum TaskType {
    #[serde(rename = "block")]
    Block,
    #[serde(rename = "pallet_balances")]
    PalletBalances,
    #[serde(rename = "execute")]
    Execute,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
enum ValueType {
    Variable(String),
    LoopVariable(String),
    Value(String),
}

impl From<String> for ValueType {
    fn from(val: String) -> Self {
        if val.contains("{{") && val.contains("}}") {
            let val = val.replace("{{", "").replace("}}", "");
            let trimmed = val.trim();

            if trimmed.starts_with("item.") {
                ValueType::LoopVariable(trimmed.replace("item.", "").to_string())
            } else {
                ValueType::Variable(trimmed.to_string())
            }
        } else {
            ValueType::Value(val)
        }
    }
}

struct VarPool {
    pool: HashMap<VariableName, serde_yaml::Value>,
}

impl VarPool {
    fn new() -> Self {
        VarPool {
            pool: HashMap::new(),
        }
    }
    fn insert(&mut self, name: VariableName, value: serde_yaml::Value) {
        self.insert(name, value)
    }
    fn get<'a>(&'a self, name: &VariableName) -> Option<&'a serde_yaml::Value> {
        self.pool.get(name)
    }
}

struct TaskList {
    tasks: Vec<Task>,
    counter: usize,
}

impl TaskList {
    fn new() -> Self {
        TaskList {
            tasks: vec![],
            counter: 0,
        }
    }
    fn add(&mut self, task: Task) {
        self.tasks.push(task);
    }
}
