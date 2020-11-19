use crate::builder::blocks::BlockCmdResult;
use crate::builder::{BlockCmd, Builder, GenesisCmd, PalletBalancesCmd};
use crate::primitives::{RawBlock, RawExtrinsic, TxtAccountSeed, TxtBlock, TxtChainSpec};
use crate::Result;
pub use processor::TaskOutcome;
use processor::{Processor, Task};
use serde::de::DeserializeOwned;

mod processor;

mapping!(
    PalletBalances => PalletBalancesCmd,
    Block => BlockCmd,
);

pub struct ToolSpec;

impl ToolSpec {
    pub fn new(yaml: &str) -> Result<()> {
        let mut proc = Processor::<Mapping>::new(yaml)?;

        for task in proc.tasks() {
            mapper(&mut proc, task)?;
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
                build:
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
