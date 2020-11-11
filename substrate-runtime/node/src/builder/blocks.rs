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

impl BlockCmd {
    pub fn build_block(txt_block: TxtBlock) -> BlockCmd {
        BlockCmd {
            call: CallCmd::BuildBlock {
                spec_block: txt_block,
            },
        }
    }
    pub fn execute_block(raw_blocks: Vec<RawBlock>) -> BlockCmd {
        BlockCmd {
            call: CallCmd::ExecuteBlocks { blocks: raw_blocks },
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockCmdResult {
    BuildBlock(RawBlock),
    ExecuteBlocks,
}

impl BlockCmd {
    pub fn run(self) -> Result<BlockCmdResult> {
        match self.call {
            CallCmd::BuildBlock { spec_block } => {
                // Convert into runtime types.
                let (at, header, extrinsics) = spec_block.prep()?;

                // Create the block by calling the runtime APIs.
                let client = ClientTemp::new()?;
                let rt = client.runtime_api();

                rt.initialize_block(&at, &header)
                    .map_err(|_| failure::err_msg(""))?;

                for extr in &extrinsics {
                    let apply_result = rt
                        .apply_extrinsic(&at, extr.clone())
                        .map_err(|_| failure::err_msg(""))?;

                    if let Err(validity) = apply_result {
                        if validity.exhausted_resources() {
                            return Err(failure::err_msg("Resources exhausted"));
                        } else {
                            return Err(failure::err_msg("Invalid transaction"));
                        }
                    } else {
                        return Err(failure::err_msg("Apply extrinsic dispatch error"));
                    }
                }

                let header = rt
                    .finalize_block(&at)
                    .map_err(|_| failure::err_msg("Failed to finalize block"))?;

                Ok(BlockCmdResult::BuildBlock(
                    Block {
                        header: header,
                        extrinsics: extrinsics,
                    }
                    .into(),
                ))
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

                Ok(BlockCmdResult::ExecuteBlocks)
            }
        }
    }
}
