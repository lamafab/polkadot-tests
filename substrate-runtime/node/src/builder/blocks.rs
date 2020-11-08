use super::primitives::{TxtBlockNumber, TxtHeader};
use super::{BlockId, Header};
use crate::executor::ClientTemp;
use crate::Result;
use sp_api::Core;
use sp_block_builder::BlockBuilder;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct BlockCmd {
    #[structopt(subcommand)]
    call: CallCmd,
}

#[derive(Debug, StructOpt)]
enum CallCmd {
    BuildBlock {
        #[structopt(short, long)]
        block_nr: TxtBlockNumber,
        #[structopt(flatten)]
        txt_header: TxtHeader,
    },
}

impl BlockCmd {
    pub fn run(&self) -> Result<()> {
        /*
        match self.call {
            CallCmd::BuildBlock { at , header } => {
                let rt = ClientTemp::new()?.runtime_api();
                //rt.initialize_block(&at, &header);
            }
        }
        */

        Ok(())
    }
}
