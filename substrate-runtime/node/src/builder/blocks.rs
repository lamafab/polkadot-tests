use super::primitives::{RawBlock, TxtBlock, TxtBlockNumber, TxtHeader};
use super::{Block, BlockId, BlockNumber, Header, UncheckedExtrinsic};
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

                for e in extrinsics {
                    let _ = rt
                        .apply_extrinsic(&at, e)
                        .map_err(|_| failure::err_msg(""))?;
                }

                rt.finalize_block(&at).map_err(|_| failure::err_msg(""))?;
            }
            CallCmd::ExecuteBlocks { blocks } => {
                // Create the block by calling the runtime APIs.
                let client = ClientTemp::new()?;
                let rt = client.runtime_api();

                // Convert into runtime native type.
                let blocks = blocks
                    .into_iter()
                    .map(|raw| Block::try_from(raw))
                    .collect::<Result<Vec<Block>>>()?;

                for mut block in blocks {
                    let at = BlockId::Hash(block.header.parent_hash.clone().try_into()?);

                    rt.execute_block(&at, block.try_into()?)
                        .map_err(|_| failure::err_msg(""))?;
                }
            }
        }

        Ok(())
    }
}
