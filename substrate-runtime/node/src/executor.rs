use sc_executor::{WasmExecutor, CallInWasm, WasmExecutionMethod};
use sc_executor::sp_wasm_interface::HostFunctions;
use sp_io::SubstrateHostFunctions;
use node_template_runtime::{WASM_BINARY, RuntimeFunction};

pub struct InitExecutor {
    exec: WasmExecutor,
    blob: Vec<u8>,
}

impl InitExecutor {
    pub fn new() -> InitExecutor {
        InitExecutor {
            exec: WasmExecutor::new(
                WasmExecutionMethod::Interpreted,
                Some(8),
                SubstrateHostFunctions::host_functions(),
                8
            ),
            blob: WASM_BINARY.expect("Wasm binary not available").to_vec(),
        }
    }
    /*
    pub fn call(&self, method: RuntimeFunction) -> Vec<u8> {
        self.exec.call_in_wasm(

        ).unwrap()
    }
    */
}
