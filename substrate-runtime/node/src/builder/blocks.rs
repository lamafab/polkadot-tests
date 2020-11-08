use super::primitives::{TxtBlock, TxtBlockNumber, TxtHeader};
use super::{BlockId, BlockNumber, Header, UncheckedExtrinsic};
use crate::executor::ClientTemp;
use crate::Result;
use sp_api::Core;
use sp_block_builder::BlockBuilder;
use std::convert::{TryFrom, TryInto};
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
}

impl BlockCmd {
    pub fn run(self) -> Result<()> {
        match self.call {
            CallCmd::BuildBlock { mut spec_block } => {
                use std::mem;

                // Convert into runtime types.
                let mut at =
                    BlockId::Hash(mem::take(&mut spec_block.header.parent_hash).try_into()?);
                let mut header = mem::take(&mut spec_block.header).try_into()?;
                let mut extrinsics = mem::take(&mut spec_block.extrinsics)
                    .into_iter()
                    .map(|e| e.try_into())
                    .collect::<Result<Vec<UncheckedExtrinsic>>>()?;

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
        }

        Ok(())
    }
}
