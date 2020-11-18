use crate::builder::blocks::BlockCmdResult;
use crate::builder::{BlockCmd, GenesisCmd, PalletBalancesCmd, Builder};
use crate::primitives::{RawBlock, RawExtrinsic, TxtAccountSeed, TxtBlock, TxtChainSpec};
use crate::Result;
pub use processor::TaskOutcome;
use processor::Processor;

mod processor;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    PalletBalances(PalletBalancesCmd),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskTypeOutcome {
    PalletBalances(<PalletBalancesCmd as Builder>::Output),
}

trait TaskRunner<TaskOutcome> {
    fn run(self) -> Result<TaskOutcome>;
}

impl TaskRunner<TaskTypeOutcome> for TaskType {
    fn run(self) -> Result<TaskTypeOutcome> {
        Ok(
            match self {
                TaskType::PalletBalances(t) => TaskTypeOutcome::PalletBalances(t.run()?)
            }
        )
    }
}

pub struct ToolSpec;

impl ToolSpec {
    #[rustfmt::skip]
    pub fn new(yaml: &str) -> Result<()> {
        let mut proc = Processor::<TaskType, TaskTypeOutcome>::new(yaml)?;

        /*
        for task in proc.tasks() {
            match task.task_type()? {
                TaskType::Block => proc.run::<TxtBlock, BlockCmdResult, _>(task, |txt_block| {
                    //BlockCmd::build_block(txt_block).run()
                    unimplemented!()
                }),
                TaskType::Execute => proc.run::<Vec<RawBlock>, BlockCmdResult, _>(task, |raw_blocks| {
                    //BlockCmd::execute_block(raw_blocks).run()
                    unimplemented!()
                }),
                TaskType::PalletBalances => proc.run::<PalletBalancesCmd, RawExtrinsic, _>(task, |call| {
                    call.run()
                }),
                TaskType::Genesis => proc.run::<Vec<TxtAccountSeed>, TxtChainSpec, _>(task, |accounts| {
                    GenesisCmd::accounts(accounts).run()
                }),
                #[cfg(test)]
                _ => panic!()
            }?;
        }
        */

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
                  parent_hash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                  number: "0x1"
                  state_root: "0x29d0d972cd27cbc511e9589fcb7a4506d5eb6a9e8df205f00472e5ab354a4e17"
                  extrinsics_root: "0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314"
                  digest:
                    logs: []
                extrinsics: []
        "#).unwrap();
    }

    #[test]
    fn pallet_balances() {
        ToolSpec::new(
            r#"
            - name: Balance transfer
              pallet_balances:
                transfer:
                  from: alice
                  to: bob
                  balance: 100
        "#,
        )
        .unwrap()
    }

    #[test]
    fn genesis() {
        ToolSpec::new(
            r#"
            - name: Create genesis
              genesis:
                - alice
                - bob
                - eve
        "#,
        )
        .unwrap()
    }
}
