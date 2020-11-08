use super::primitives::{RawBlock, TxtBlock, TxtBlockNumber, TxtHeader};
use super::{BlockId, BlockNumber, Header, UncheckedExtrinsic};
use crate::executor::ClientTemp;
use crate::Result;
use sp_api::Core;
use sp_block_builder::BlockBuilder;
use std::convert::{TryFrom, TryInto};
use std::mem;
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
        #[structopt(flatten)]
        spec_block: TxtBlock,
    },
    ExecuteBlocks {
        #[structopt(short, long)]
        blocks: Vec<RawBlock>,
    },
}

impl BlockCmd {
    pub fn run(self) -> Result<()> {
        match self.call {
            CallCmd::BuildBlock { mut spec_block } => {
                // Convert into runtime types.
                let (at, header, extrinsics) = spec_block.prep()?;

                // Create the block by calling the runtime APIs.
                let client = ClientTemp::new()?;
                let rt = client.runtime_api();

                rt.initialize_block(&at, &header)
                    .map_err(|_| failure::err_msg(""))?;

                extrinsics
                    .into_iter()
                    .map(|e| {
                        rt.apply_extrinsic(&at, e)
                            .map(|_| ())
                            .map_err(|_| failure::err_msg(""))
                    })
                    .collect::<Result<Vec<()>>>()?;

                rt.finalize_block(&at).map_err(|_| failure::err_msg(""))?;
            }
            CallCmd::ExecuteBlocks { blocks } => {}
        }

        Ok(())
    }
}
