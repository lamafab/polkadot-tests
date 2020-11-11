use crate::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cell::Cell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::drop;

pub struct Parser {
    global_var_pool: VarPool,
    tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome<T> {
    name: String,
    data: Vec<T>,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self> {
        let (global_var_pool, tasks) = global_parser(input)?;

        Ok(Parser {
            global_var_pool: global_var_pool,
            tasks: tasks,
        })
    }
    pub fn tasks(&self) -> &Vec<Task> {
        &self.tasks
    }
    pub fn run<T: DeserializeOwned, R: Serialize, F: Fn(T) -> Result<R>>(
        &self,
        task: &Task,
        f: F,
    ) -> Result<()> {
        let tasks = task_parser(&self.global_var_pool, &task.properties)?;
        let mut results = vec![];

        for task in tasks {
            results.push(f(task)?);
        }

        println!(
            "{}",
            serde_json::to_string_pretty(&Outcome {
                name: task.name().to_string(),
                data: results,
            })?
        );

        Ok(())
    }
}

fn global_parser(input: &str) -> Result<(VarPool, Vec<Task>)> {
    let yaml_blocks: Vec<YamlItem> = serde_yaml::from_str(input)?;

    let mut tasks = vec![];
    let mut global_vars = None;
    let mut global_var_pool = VarPool::new();

    // A "local" variable pool is not relevant in this context.
    let converter = PrimitiveConverter::new(&global_var_pool, &global_var_pool, 0);

    // Process global variables.
    let mut first_vars = false;
    for item in yaml_blocks {
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
    let mut _register = false;

    let mut local_var_pool = VarPool::new();
    let converter = PrimitiveConverter::new(global_var_pool, &local_var_pool, 0);

    let mut vars = None;
    let mut loop_vars = None;

    for (key, val) in properties {
        match key {
            KeyType::TaskType(_) => {}
            KeyType::Keyword(keyword) => match keyword {
                Keyword::Register => _register = true,
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
    #[rustfmt::skip]
    fn process_yaml_value(&self, value: &mut serde_yaml::Value) -> Result<()> {
        if let Some(v) = value.as_str() {
            if v.contains("{{") && v.contains("}}") {
                let v = v.replace("{{", "").replace("}}", "");
                let var_name = VariableName(v.trim().to_string());

                let var = if let Some(var) = self.local_var_pool.get(self.loop_index, &var_name) {
                    var
                } else if let Some(var) = self.global_var_pool.get(self.loop_index, &var_name) {
                    var
                } else {
                    return Err(failure::err_msg(format!(
                        "Variable \"{}\" not found",
                        var_name.0
                    )));
                };

                *value = var;
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

struct NestedVariable<'a> {
    names: Vec<VariableName>,
    index: Cell<usize>,
    p: PhantomData<&'a ()>,
}

impl<'a> NestedVariable<'a> {
    fn new(name: &'a VariableName) -> Self {
        NestedVariable {
            names: name
                .0
                .split(".")
                .map(|s| VariableName(s.to_string()))
                .collect(),
            index: Cell::new(0),
            p: PhantomData,
        }
    }
    fn incr_index(&self) {
        // `Cell::update()` is not stable yet.
        let current = self.index.get();
        self.index.set(current + 1);
    }
    fn is_loop(&self) -> bool {
        self.names
            .first()
            .map(|v| {
                if v.as_str() == "item" {
                    self.incr_index();
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }
    fn name(&'a self) -> Option<&'a VariableName> {
        let index = self.index.get();
        self.names.get(index).map(|v| {
            self.incr_index();
            v
        })
    }
    fn generic_name(&self) -> Option<serde_yaml::Value> {
        self.name().map(|n| serde_yaml::Value::from(n.as_str()))
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
    fn insert_loop(&mut self, pool: LoopType) {
        self.loop_pool = pool;
    }
    fn get<'a>(&'a self, index: usize, name: &'a VariableName) -> Option<serde_yaml::Value> {
        let nested = NestedVariable::new(name);
        if nested.is_loop() {
            Self::search_nested(&nested, self.loop_pool.0.get(index)?).map(|v| v.clone())
        } else {
            Self::search_nested(&nested, self.pool.0.get(nested.name()?)?).map(|v| v.clone())
        }
    }
    fn search_nested<'a>(
        nested_var: &'a NestedVariable<'a>,
        value: &'a serde_yaml::Value,
    ) -> Option<&'a serde_yaml::Value> {
        if let Some(name) = nested_var.generic_name() {
            if let Some(map) = value.as_mapping() {
                if let Some(v) = map.get(&name) {
                    Self::search_nested(nested_var, v)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            Some(value)
        }
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct VariableName(String);

impl VariableName {
    fn as_str<'a>(&'a self) -> &'a str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    name: String,
    #[serde(flatten)]
    properties: HashMap<KeyType, serde_yaml::Value>,
}

impl Task {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn task_type(&self) -> Result<&TaskType> {
        let mut task = None;

        for (key, _) in &self.properties {
            match key {
                KeyType::TaskType(task_ty) => {
                    if task.is_none() {
                        task = Some(task_ty)
                    } else {
                        return Err(failure::err_msg(
                            "Only one task type per yaml block allowed",
                        ));
                    }
                }
                _ => {}
            }
        }

        Ok(task.ok_or(failure::err_msg("No task found for yaml block"))?)
    }
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
pub enum TaskType {
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
                categories: vec!["business".to_string(), "finance".to_string()]
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
                categories: vec!["business".to_string(), "finance".to_string()]
            }
        );
    }

    #[test]
    fn converter_local_vars_map_abbreviated_nested() {
        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: usize,
            categories: Vec<String>,
            employer: String,
        }

        let yaml = r#"
            - name: Some person
              person:
                name: alice
                age: "{{ info.age }}"
                categories: "{{ info.categories }}"
                employer: "{{ info.job.employer}}"
              vars:
                info: {
                  age: 33,
                  categories: ["business", "finance"],
                  job: {
                    position: "accountant",
                    employer: "CorpA"
                  }
                }
        "#;

        let res = parse::<Person>(yaml);
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0],
            Person {
                name: "alice".to_string(),
                age: 33,
                categories: vec!["business".to_string(), "finance".to_string()],
                employer: "CorpA".to_string(),
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
                categories: vec!["business".to_string(), "finance".to_string()]
            }
        );
        assert_eq!(
            res[1],
            Person {
                name: "bob".to_string(),
                age: 33,
                categories: vec!["business".to_string(), "finance".to_string()]
            }
        );
        assert_eq!(
            res[2],
            Person {
                name: "eve".to_string(),
                age: 33,
                categories: vec!["business".to_string(), "finance".to_string()]
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
                categories: vec!["business".to_string(), "finance".to_string()]
            }
        );
        assert_eq!(
            res[1],
            Person {
                name: "alice".to_string(),
                age: 33,
                categories: vec!["hr".to_string(), "marketing".to_string()]
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

    #[test]
    fn converter_loop_map_abbreviated_nested() {
        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: usize,
            hair: String,
            height: usize,
        }

        let yaml = r#"
            - name: Some person
              person:
                name: alice
                age: 33
                hair: "{{ item.hair }}"
                height: "{{ item.height }}"
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
                hair: "blonde".to_string(),
                height: 174,
            }
        );
        assert_eq!(
            res[1],
            Person {
                name: "alice".to_string(),
                age: 33,
                hair: "red".to_string(),
                height: 165,
            }
        );
    }
}
