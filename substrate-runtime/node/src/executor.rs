use sc_executor::{WasmExecutor, CallInWasm, WasmExecutionMethod};
use sc_executor::sp_wasm_interface::HostFunctions;
use sp_io::SubstrateHostFunctions;

pub struct InitExecutor {
    exec: WasmExecutor,
}

impl InitExecutor {
    pub fn new() -> InitExecutor {
        InitExecutor {
            exec: WasmExecutor::new(
                WasmExecutionMethod::Interpreted,
                Some(8),
                SubstrateHostFunctions::host_functions(),
                8
            )
        }
    }
}
