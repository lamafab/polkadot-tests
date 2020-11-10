use crate::executor::ClientTemp;
use crate::primitives::runtime::{Block, BlockId};
use crate::primitives::{RawBlock, TxtBlock};
use crate::Result;
use sp_api::Core;
use sp_block_builder::BlockBuilder;
use std::convert::{TryFrom, TryInto};
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
            CallCmd::BuildBlock { spec_block } => {
                // Convert into runtime types.
                let (at, header, extrinsics) = spec_block.prep()?;

                // Create the block by calling the runtime APIs.
                let client = ClientTemp::new()?;
                let rt = client.runtime_api();

                rt.initialize_block(&at, &header)
                    .map_err(|_| failure::err_msg(""))?;

                for extr in extrinsics {
                    let apply_result = rt
                        .apply_extrinsic(&at, extr)
                        .map_err(|_| failure::err_msg(""))?;

                    if let Err(validity) = apply_result {
                        if validity.exhausted_resources() {
                            break;
                        } else {
                            return Err(failure::err_msg("Invalid transaction"));
                        }
                    } else {
                        return Err(failure::err_msg("Apply extrinsic dispatch error"));
                    }
                }

                rt.finalize_block(&at)
                    .map_err(|_| failure::err_msg("Failed to finalize block"))?;
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

                for block in blocks {
                    let at = BlockId::Hash(block.header.parent_hash.clone().try_into()?);

                    rt.execute_block(&at, block.try_into()?)
                        .map_err(|_| failure::err_msg(""))?;
                }
            }
        }

        Ok(())
    }
}
