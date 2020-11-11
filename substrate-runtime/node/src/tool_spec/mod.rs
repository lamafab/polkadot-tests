use crate::Result;
use parser::{Parser, TaskType};
use std::fs;

mod parser;

pub struct ToolSpec {}

impl ToolSpec {
    pub fn new_from_file(path: &str) -> Result<()> {
        let yaml = fs::read_to_string(path)?;
        let parser = Parser::new(&yaml)?;

        for task in parser.tasks() {
            match task.task_type()? {
                TaskType::Block => unimplemented!(),
                TaskType::Execute => unimplemented!(),
                TaskType::PalletBalances => unimplemented!(),
            }
        }

        Ok(())
    }
}
