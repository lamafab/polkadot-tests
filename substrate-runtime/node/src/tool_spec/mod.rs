use crate::Result;
use std::collections::HashMap;
use std::fs;

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

        Ok(ToolSpec {
            var_pool: var_pool,
            task_list: task_list,
        })
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum VarsType {
    Map(HashMap<VariableName, serde_yaml::Value>),
    List(Vec<serde_yaml::Value>),
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
        TaskList { tasks: vec![] }
    }
}
