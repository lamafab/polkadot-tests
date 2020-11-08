use super::primitives::{TxtBlockNumber, TxtHeader};
use super::{BlockNumber, BlockId, Header};
use crate::executor::ClientTemp;
use crate::Result;
use sp_api::Core;
use sp_block_builder::BlockBuilder;
use structopt::StructOpt;
use std::str::FromStr;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, StructOpt)]
pub struct BlockCmd {
    #[structopt(subcommand)]
    call: CallCmd,
}

#[derive(Debug, StructOpt)]
enum CallCmd {
    BuildBlock {
        #[structopt(flatten)]
        header: TxtHeader,
    },
}

impl BlockCmd {
    pub fn run(self) -> Result<()> {
        match self.call {
            CallCmd::BuildBlock { header } => {
                let header: Header = header.try_into()?;
                let at = BlockId::Hash(header.parent_hash.clone());

                let client = ClientTemp::new()?;
                let rt = client.runtime_api();
                rt.initialize_block(&at, &header);
            }
        }

        Ok(())
    }
}
