use crate::Result;
use std::collections::HashMap;
use std::fs;
use std::mem::drop;
use serde::de::DeserializeOwned;

mod block;

pub struct ToolSpec {
    var_pool: VarPool,
    task_list: TaskList,
}

impl ToolSpec {
    pub fn new_from_file(path: &str) -> Result<Self> {
        let yaml_file = fs::read_to_string(path)?;
        let mut yaml_blocks: Vec<YamlItem> = serde_yaml::from_str(&yaml_file)?;

        let mut var_pool = VarPool::new();
        let mut task_list = TaskList::new();

        let mut global_var_pool = VarPool::new();
        let mut global_vars = None;
        // A local variable pool is not relevant in this context, since we're looking for global variables.
        let converter = PrimitiveConverter::new(&global_var_pool, &global_var_pool, 0);

        // Process global variables first.
        let mut first_vars = false;
        for mut item in yaml_blocks {
            match item {
                YamlItem::Vars(mut vars) => {
                    if !first_vars {
                        for (_, var) in &mut vars.vars.0 {
                            converter.process_yaml_value(var)?;
                        }

                        global_vars = Some(vars.vars);
                        first_vars = true;
                    } else {
                        return Err(failure::err_msg(
                            "Only one global variable entry block allowed",
                        ));
                    }
                }
                _ => {}
            }
        }

        drop(converter);

        if let Some(vars) = global_vars {
            global_var_pool.insert(vars);
        }

        Ok(ToolSpec {
            var_pool: var_pool,
            task_list: task_list,
        })
    }
    pub fn run(self) {

    }
}

struct TaskOutcome<T> {
    name: String,
    out: Box<T>,
}

fn task_parser<T: DeserializeOwned>(global_var_pool: &VarPool, mut properties: HashMap<KeyType, serde_yaml::Value>) -> Result<()> {
    let mut task = None;
    let mut register = false;
    let mut first_loop = false;
    let mut first_vars = false;

    let mut local_var_pool = VarPool::new();
    let converter = PrimitiveConverter::new(global_var_pool, &local_var_pool, 0);

    let mut vars = None;
    let mut loop_vars = None;

    for (key, val) in &properties {
        match key {
            KeyType::TaskType(task_ty) => {
                if task.is_none() {
                    task = Some(task_ty)
                } else {
                    return Err(failure::err_msg("Only one task per yaml block is allowed"));
                }
            }
            KeyType::Keyword(keyword) => match keyword {
                Keyword::Register => register = true,
                Keyword::Loop => {
                    if !first_loop {
                        first_loop = true;
                        let mut parsed = serde_yaml::from_value::<LoopType>(val.clone())?;
                        for v in &mut parsed.0 {
                            converter.process_yaml_value(v)?;
                        }

                        loop_vars = Some(parsed);
                    } else {
                        return Err(failure::err_msg("Only one loop entry per task allowed"));
                    }
                }
                Keyword::Vars => {
                    if !first_vars {
                        first_vars = true;
                        let mut parsed = serde_yaml::from_value::<VarType>(val.clone())?;
                        for (_, v) in &mut parsed.0 {
                            converter.process_yaml_value(v)?;
                        }

                        vars = Some(parsed);
                    } else {
                        return Err(failure::err_msg(
                            "Only one variable entry per task allowed ",
                        ));
                    }
                }
            },
        }
    }

    // Drop converter so variables can be inserted.
    drop(converter);

    if let Some(vars) = vars {
        local_var_pool.insert(vars);
    }

    if let Some(vars) = loop_vars {
        local_var_pool.insert_loop(vars);
    }

    let converter = PrimitiveConverter::new(global_var_pool, &local_var_pool, 0);
    converter.process_properties(&mut properties)?;

    Ok(())
}

struct PrimitiveConverter<'a> {
    global_var_pool: &'a VarPool,
    local_var_pool: &'a VarPool,
    loop_index: usize,
}

impl<'a> PrimitiveConverter<'a> {
    fn new(global: &'a VarPool, local: &'a VarPool, index: usize) -> Self {
        PrimitiveConverter {
            global_var_pool: global,
            local_var_pool: local,
            loop_index: index,
        }
    }
    fn process_properties(&self, properties: &mut HashMap<KeyType, serde_yaml::Value>) -> Result<()> {
        for (_, val) in properties {
            self.process_yaml_value(val)?;
        }

        Ok(())
    }
    fn process_value_ty(&self, val_ty: &mut ValType) -> Result<()> {
        match val_ty {
            ValType::List(list) => {
                for v in list {
                    self.process_value_ty(v);
                }
            }
            ValType::Map(map) => {
                for (_, v) in map {
                    self.process_value_ty(v)?;
                }
            }
            ValType::SingleValue(val) => self.process_yaml_value(val)?,
        }

        Ok(())
    }
    fn process_yaml_value(&self, value: &mut serde_yaml::Value) -> Result<()> {
        if let Some(v) = value.as_str() {
            if v.contains("{{") && v.contains("}}") {
                let v = v.replace("{{", "").replace("}}", "");
                let var_name = VariableName(v.trim().to_string());

                if var_name.0.starts_with("item") {
                    if let Some(var) = self.local_var_pool.get_loop(self.loop_index, &var_name) {
                        *value = var.clone();
                    } else if let Some(var) =
                        self.global_var_pool.get_loop(self.loop_index, &var_name)
                    {
                        *value = var.clone();
                    } else {
                        return Err(failure::err_msg(format!(
                            "Variable \"{}\" not found",
                            var_name.0
                        )));
                    }
                } else {
                    if let Some(var) = self.local_var_pool.get(&var_name) {
                        *value = var.clone();
                    } else if let Some(var) = self.global_var_pool.get(&var_name) {
                        *value = var.clone();
                    } else {
                        return Err(failure::err_msg(format!(
                            "Variable \"{}\" not found",
                            var_name.0
                        )));
                    }
                }

                self.process_yaml_value(value)?;
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
    vars: VarType,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
struct VarType(HashMap<VariableName, serde_yaml::Value>);

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
struct LoopType(Vec<serde_yaml::Value>);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum ValType {
    List(Vec<ValType>),
    Map(HashMap<String, ValType>),
    SingleValue(serde_yaml::Value),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct VariableName(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    name: String,
    #[serde(flatten)]
    properties: HashMap<KeyType, serde_yaml::Value>,
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

struct VarPool {
    pool: VarType,
    loop_pool: LoopType,
}

impl VarPool {
    fn new() -> Self {
        VarPool {
            pool: Default::default(),
            loop_pool: Default::default(),
        }
    }
    fn insert(&mut self, vars: VarType) {
        for (name, val) in vars.0 {
            self.pool.0.insert(name, val);
        }
    }
    fn insert_loop(&mut self, mut pool: LoopType) {
        self.loop_pool = pool;
    }
    fn get<'a>(&'a self, name: &VariableName) -> Option<&'a serde_yaml::Value> {
        self.pool.0.get(name)
    }
    fn get_loop<'a>(&'a self, index: usize, _name: &VariableName) -> Option<&'a serde_yaml::Value> {
        self.loop_pool.0.get(index)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converter() {
        let yaml = r#"
        
        "#;
    }
}
