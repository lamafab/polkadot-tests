use super::builder::Block;
use super::service::Executor;
use super::Result;
use node_template_runtime::{RuntimeFunction, WASM_BINARY};
use sc_client_api::in_mem::Backend;
use sc_executor::sp_wasm_interface::HostFunctions;
use sc_executor::{CallInWasm, NativeExecutor, WasmExecutionMethod, WasmExecutor};
use sc_service::client::{new_in_mem, Client, ClientConfig, LocalCallExecutor};
use sp_core::testing::TaskExecutor;
use sp_core::traits::MissingHostFunctions;
use sp_io::{SubstrateHostFunctions, TestExternalities};
use sp_storage::Storage;
use std::sync::Arc;

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
    pub fn call(
        &mut self,
        func: RuntimeFunction,
        data: &[u8],
    ) -> std::result::Result<Vec<u8>, String> {
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

pub struct ClientTemp<RA> {
    client: Client<
        Backend<Block>,
        LocalCallExecutor<Backend<Block>, NativeExecutor<Executor>>,
        Block,
        RA,
    >,
}

impl<RA> ClientTemp<RA> {
    pub fn new() -> Result<ClientTemp<RA>> {
        Ok(ClientTemp {
            client: new_in_mem::<_, Block, _, _>(
                NativeExecutor::<Executor>::new(WasmExecutionMethod::Interpreted, None, 8),
                &Storage::default(),
                None,
                None,
                Box::new(TaskExecutor::new()),
                ClientConfig::default(),
            )
            .map_err(|_| failure::err_msg("failed to create in-memory client"))?,
        })
    }
}
