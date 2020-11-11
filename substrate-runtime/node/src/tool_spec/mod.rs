use crate::Result;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fs;
use std::mem::drop;

mod block;

pub struct ToolSpec {
    var_pool: VarPool,
}

impl ToolSpec {
    pub fn new_from_file(path: &str) -> Result<Self> {
        let yaml_file = fs::read_to_string(path)?;
        let mut yaml_blocks: Vec<YamlItem> = serde_yaml::from_str(&yaml_file)?;

        let mut var_pool = VarPool::new();

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

        Ok(ToolSpec { var_pool: var_pool })
    }
    pub fn run(self) {}
}

struct TaskOutcome<T> {
    name: String,
    out: Box<T>,
}

fn global_parser(input: &str) -> Result<(VarPool, Vec<Task>)> {
    let mut yaml_blocks: Vec<YamlItem> = serde_yaml::from_str(input)?;

    let mut tasks = vec![];
    let mut global_vars = None;
    let mut global_var_pool = VarPool::new();

    // A "local" variable pool is not relevant in this context.
    let converter = PrimitiveConverter::new(&global_var_pool, &global_var_pool, 0);

    // Process global variables.
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
            YamlItem::Task(task) => tasks.push(task),
        }
    }

    // Drop converter so variables can be inserted into pool.
    drop(converter);

    if let Some(vars) = global_vars {
        global_var_pool.insert(vars);
    }

    Ok((global_var_pool, tasks))
}

fn task_parser<T: DeserializeOwned>(
    global_var_pool: &VarPool,
    properties: &HashMap<KeyType, serde_yaml::Value>,
) -> Result<Vec<T>> {
    let mut register = false;

    let mut local_var_pool = VarPool::new();
    let converter = PrimitiveConverter::new(global_var_pool, &local_var_pool, 0);

    let mut vars = None;
    let mut loop_vars = None;

    for (key, val) in properties {
        match key {
            KeyType::TaskType(task_ty) => {}
            KeyType::Keyword(keyword) => match keyword {
                Keyword::Register => register = true,
                Keyword::Loop => {
                    // Ensure only one `loop:` entry is present per task.
                    if loop_vars.is_none() {
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
                    // Ensure only one `vars:` entry is present per task.
                    if vars.is_none() {
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

    // Keep track of loop count
    let loop_count = loop_vars.as_ref().map(|l| l.len()).unwrap_or(1);
    println!(">> {}", loop_count);

    // Drop converter so variables can be inserted into pool.
    drop(converter);

    if let Some(vars) = vars {
        local_var_pool.insert(vars);
    }

    if let Some(vars) = loop_vars {
        local_var_pool.insert_loop(vars);
    }

    let mut expanded = vec![];

    for index in 0..loop_count {
        let mut loop_properties = properties.clone();
        let converter = PrimitiveConverter::new(global_var_pool, &local_var_pool, index);
        converter.process_properties(&mut loop_properties)?;

        for (key, val) in loop_properties {
            match key {
                KeyType::TaskType(_) => {
                    expanded.push(serde_yaml::from_value::<T>(val)?);
                }
                _ => {}
            }
        }
    }

    Ok(expanded)
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
    fn process_properties(
        &self,
        properties: &mut HashMap<KeyType, serde_yaml::Value>,
    ) -> Result<()> {
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
    #[rustfmt::skip]
    fn process_yaml_value(&self, value: &mut serde_yaml::Value) -> Result<()> {
        if let Some(v) = value.as_str() {
            if v.contains("{{") && v.contains("}}") {
                let v = v.replace("{{", "").replace("}}", "");
                let var_name = VariableName(v.trim().to_string());

                let var = if var_name.0.starts_with("item") {
                    if let Some(var) = self.local_var_pool.get_loop(self.loop_index, &var_name) {
                        var
                    } else if let Some(var) = self.global_var_pool.get_loop(self.loop_index, &var_name) {
                        var
                    } else {
                        return Err(failure::err_msg(format!(
                            "Variable \"{}\" not found",
                            var_name.0
                        )));
                    }
                } else {
                    if let Some(var) = self.local_var_pool.get(&var_name) {
                        var
                    } else if let Some(var) = self.global_var_pool.get(&var_name) {
                        var
                    } else {
                        return Err(failure::err_msg(format!(
                            "Variable \"{}\" not found",
                            var_name.0
                        )));
                    }
                };

                *value = var.clone();

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

impl LoopType {
    fn len(&self) -> usize {
        self.0.len()
    }
}

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
    #[cfg(test)]
    #[serde(rename = "person")]
    Person,
    #[cfg(test)]
    #[serde(rename = "animal")]
    Animal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;

    fn parse<T: DeserializeOwned>(input: &str) -> Vec<T> {
        let (var_pool, tasks) = global_parser(input).unwrap();
        task_parser(&var_pool, &tasks[0].properties).unwrap()
    }

    #[test]
    fn converter_simple() {
        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: usize,
            categories: Vec<String>,
        }

        let yaml = r#"
            - name: Some person
              person:
                name: alice
                age: 33
                categories:
                  - business
                  - finance
        "#;

        let res = parse::<Person>(yaml);
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0],
            Person {
                name: "alice".to_string().to_string(),
                age: 33,
                categories: vec!["business".to_string(), "finance".to_string(),]
            }
        );
    }

    #[test]
    fn converter_local_vars() {
        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: usize,
            categories: Vec<String>,
        }

        let yaml = r#"
            - name: Some person
              person:
                name: alice
                age: "{{ age }}"
                categories:
                  - business
                  - finance
              vars:
                age: 33
        "#;

        let res = parse::<Person>(yaml);
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0],
            Person {
                name: "alice".to_string(),
                age: 33,
                categories: vec!["business".to_string(), "finance".to_string(),]
            }
        );
    }

    #[test]
    fn converter_loop_string() {
        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: usize,
            categories: Vec<String>,
        }

        let yaml = r#"
            - name: Some person
              person:
                name: "{{ item }}"
                age: 33
                categories:
                  - business
                  - finance
              loop:
                - alice
                - bob
                - eve
        "#;

        let res = parse::<Person>(yaml);
        assert_eq!(res.len(), 3);
        assert_eq!(
            res[0],
            Person {
                name: "alice".to_string(),
                age: 33,
                categories: vec!["business".to_string(), "finance".to_string(),]
            }
        );
        assert_eq!(
            res[1],
            Person {
                name: "bob".to_string(),
                age: 33,
                categories: vec!["business".to_string(), "finance".to_string(),]
            }
        );
        assert_eq!(
            res[2],
            Person {
                name: "eve".to_string(),
                age: 33,
                categories: vec!["business".to_string(), "finance".to_string(),]
            }
        );
    }

    #[test]
    fn converter_loop_list() {
        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: usize,
            categories: Vec<String>,
        }

        let yaml = r#"
            - name: Some person
              person:
                name: alice
                age: 33
                categories: "{{ item }}"
              loop:
                -
                  - business
                  - finance
                -
                  - hr
                  - marketing
        "#;

        let res = parse::<Person>(yaml);
        assert_eq!(res.len(), 2);
        assert_eq!(
            res[0],
            Person {
                name: "alice".to_string(),
                age: 33,
                categories: vec!["business".to_string(), "finance".to_string(),]
            }
        );
        assert_eq!(
            res[1],
            Person {
                name: "alice".to_string(),
                age: 33,
                categories: vec!["hr".to_string(), "marketing".to_string(),]
            }
        );
    }

    #[test]
    fn converter_loop_map() {
        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: usize,
            attributes: Attributes,
        }

        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Attributes {
            hair: String,
            height: usize,
        }

        let yaml = r#"
            - name: Some person
              person:
                name: alice
                age: 33
                attributes: "{{ item }}"
              loop:
                -
                  hair: blonde
                  height: 174
                -
                  hair: red
                  height: 165

        "#;

        let res = parse::<Person>(yaml);
        assert_eq!(res.len(), 2);
        assert_eq!(
            res[0],
            Person {
                name: "alice".to_string(),
                age: 33,
                attributes: Attributes {
                    hair: "blonde".to_string(),
                    height: 174
                }
            }
        );
        assert_eq!(
            res[1],
            Person {
                name: "alice".to_string(),
                age: 33,
                attributes: Attributes {
                    hair: "red".to_string(),
                    height: 165
                }
            }
        );
    }

    #[test]
    fn converter_loop_map_abbreviated() {
        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: usize,
            attributes: Attributes,
        }

        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Attributes {
            hair: String,
            height: usize,
        }

        let yaml = r#"
            - name: Some person
              person:
                name: alice
                age: 33
                attributes: "{{ item }}"
              loop:
                - { hair: "blonde", height: 174 }
                - { hair: "red", height: 165 }
        "#;

        let res = parse::<Person>(yaml);
        assert_eq!(res.len(), 2);
        assert_eq!(
            res[0],
            Person {
                name: "alice".to_string(),
                age: 33,
                attributes: Attributes {
                    hair: "blonde".to_string(),
                    height: 174
                }
            }
        );
        assert_eq!(
            res[1],
            Person {
                name: "alice".to_string(),
                age: 33,
                attributes: Attributes {
                    hair: "red".to_string(),
                    height: 165
                }
            }
        );
    }
}
