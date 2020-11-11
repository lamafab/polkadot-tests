use crate::builder::balances::TransferDetails;
use crate::builder::blocks::BlockCmdResult;
use crate::builder::{BlockCmd, PalletBalancesCmd};
use crate::primitives::{RawBlock, RawExtrinsic, TxtBlock};
use crate::Result;
use parser::{Parser, TaskType};

mod parser;

pub struct ToolSpec;

impl ToolSpec {
    #[rustfmt::skip]
    pub fn new(yaml: &str) -> Result<()> {
        let parser = Parser::new(yaml)?;

        for task in parser.tasks() {
            match task.task_type()? {
                TaskType::Block => parser.run::<TxtBlock, BlockCmdResult, _>(task, |txt_block| {
                    BlockCmd::build_block(txt_block).run()
                }),
                TaskType::Execute => parser.run::<Vec<RawBlock>, BlockCmdResult, _>(task, |raw_blocks| {
                    BlockCmd::execute_block(raw_blocks).run()
                }),
                TaskType::PalletBalances => parser.run::<TransferDetails, RawExtrinsic, _>(task, |details| {
                        PalletBalancesCmd::transfer(details).run()
                    }),
                #[cfg(test)]
                _ => panic!()
            }?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_block() {
        ToolSpec::new(r#"
            - name: Build block
              block:
                header:
                  parent_hash: "0000000000000000000000000000000000000000000000000000000000000000"
                  number: "44"
                  state_root: "29d0d972cd27cbc511e9589fcb7a4506d5eb6a9e8df205f00472e5ab354a4e17"
                  extrinsics_root: "03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314"
                  digest:
                    logs: []
                extrinsics: []
        "#).unwrap();
    }
}
