use crate::Result;
use std::collections::HashMap;
use std::fs;
use std::mem::drop;

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
    pub fn run(self) {}
}

struct TaskOutcome<T> {
    name: String,
    out: Box<T>,
}

fn task_parser(properties: HashMap<KeyType, serde_yaml::Value>) -> Result<()> {
    let mut task = None;
    let mut local_var_pool = VarPool::new();

    let mut register = false;
    let mut first_loop = false;
    let mut first_vars = false;

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
                        local_var_pool.insert(serde_yaml::from_value(val.clone())?);
                    } else {
                        return Err(failure::err_msg("Only one loop entry per task allowed"));
                    }
                }
                Keyword::Vars => {
                    if !first_vars {
                        first_vars = true;
                        local_var_pool.insert(serde_yaml::from_value(val.clone())?);
                    } else {
                        return Err(failure::err_msg(
                            "Only one variable entry per task allowed ",
                        ));
                    }
                }
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
    fn process_properties(&self, properties: &mut HashMap<String, ValType>) -> Result<()> {
        for (_, val) in properties {
            self.process_value_ty(val)?;
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

                if var_name.0.starts_with("item.") {
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct VarType(HashMap<VariableName, serde_yaml::Value>);

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
    pool: HashMap<VariableName, serde_yaml::Value>,
    loop_pool: Vec<HashMap<VariableName, serde_yaml::Value>>,
}

impl VarPool {
    fn new() -> Self {
        VarPool {
            pool: HashMap::new(),
            loop_pool: vec![],
        }
    }
    fn insert(&mut self, vars: VarType) {
        for (name, val) in vars.0 {
            self.pool.insert(name, val);
        }
    }
    fn insert_loop(&mut self, pool: HashMap<VariableName, serde_yaml::Value>) {
        self.loop_pool.push(pool);
    }
    fn get<'a>(&'a self, name: &VariableName) -> Option<&'a serde_yaml::Value> {
        self.pool.get(name)
    }
    fn get_loop<'a>(&'a self, index: usize, name: &VariableName) -> Option<&'a serde_yaml::Value> {
        self.loop_pool.get(index).unwrap().get(name)
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
