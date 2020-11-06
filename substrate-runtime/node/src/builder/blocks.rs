use crate::executor::ClientTemp;
use crate::Result;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct BlockCmd {
    #[structopt(subcommand)]
    call: CallCmd,
}

#[derive(Debug, StructOpt)]
enum CallCmd {
    BuildBlock,
}

impl BlockCmd {
    pub fn run(&self) -> Result<()> {
        match self.call {
            CallCmd::BuildBlock => {
                ClientTemp::new()?.exec_context(|| Ok(Option::<()>::None));
            }
        }

        Ok(())
    }
}
