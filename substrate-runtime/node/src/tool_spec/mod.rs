use crate::builder::balances::TransferDetails;
use crate::builder::blocks::BlockCmdResult;
use crate::builder::{BlockCmd, PalletBalancesCmd};
use crate::primitives::{RawBlock, RawExtrinsic, TxtBlock};
use crate::Result;
use parser::{Parser, TaskType};
use std::fs;

mod parser;

pub struct ToolSpec {}

impl ToolSpec {
    #[rustfmt::skip]
    pub fn new_from_file(path: &str) -> Result<()> {
        let yaml = fs::read_to_string(path)?;
        let parser = Parser::new(&yaml)?;

        for task in parser.tasks() {
            match task.task_type()? {
                TaskType::Block => parser.run::<TxtBlock, BlockCmdResult, _>(task, |txt_block| {
                    BlockCmd::build_block(txt_block).run()
                }),
                TaskType::Execute => parser.run::<Vec<RawBlock>, BlockCmdResult, _>(task, |raw_blocks| {
                    BlockCmd::execute_block(raw_blocks).run()
                }),
                TaskType::PalletBalances => parser.run::<TransferDetails, RawExtrinsic, _>(task, |details| {
                        PalletBalancesCmd::transer(details).run()
                    }),
            }?;
        }

        Ok(())
    }
}
