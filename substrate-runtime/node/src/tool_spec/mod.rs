use crate::builder::{BlockCmd, PalletBalancesCmd};

use crate::Result;
use processor::{Processor, Task};

use std::cmp::PartialEq;
use std::hash::Hash;

mod processor;
pub use processor::{Mapper, TaskOutcome};

mapping!(
    PalletBalances => PalletBalancesCmd,
    Block => BlockCmd,
);

pub fn run_tool_spec(yaml: &str) -> Result<()> {
    Processor::<Mapping>::new(yaml)?.process()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_block() {
        run_tool_spec(r#"
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
        run_tool_spec(
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

    /*
    #[test]
    fn genesis() {
        run_tool_spec(
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
    */
}
