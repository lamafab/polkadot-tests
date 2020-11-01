use node_template_runtime::{RuntimeFunction, WASM_BINARY};
use sc_executor::sp_wasm_interface::HostFunctions;
use sc_executor::{CallInWasm, WasmExecutionMethod, WasmExecutor};
use sp_core::traits::MissingHostFunctions;
use sp_io::{SubstrateHostFunctions, TestExternalities};

pub struct InitExecutor {
    exec: WasmExecutor,
    blob: Vec<u8>,
    ext: TestExternalities,
}

impl InitExecutor {
    pub fn new() -> InitExecutor {
        InitExecutor {
            exec: WasmExecutor::new(
                WasmExecutionMethod::Interpreted,
                Some(8),
                SubstrateHostFunctions::host_functions(),
                8,
            ),
            blob: WASM_BINARY.expect("Wasm binary not available").to_vec(),
            ext: TestExternalities::default(),
        }
    }
    pub fn call(&mut self, func: RuntimeFunction, data: &[u8]) -> Result<Vec<u8>, String> {
        self.exec.call_in_wasm(
            &self.blob,
            None,
            func.as_str(),
            data,
            &mut self.ext.ext(),
            MissingHostFunctions::Disallow,
        )
    }
}
