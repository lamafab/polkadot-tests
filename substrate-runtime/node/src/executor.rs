use super::chain_spec::{gen_chain_spec_thin, ChainSpec};
use super::Result;
use crate::primitives::runtime::Block;
use node_template_runtime::{RuntimeApi, RuntimeApiImpl, RuntimeFunction, WASM_BINARY};
use sc_client_api::in_mem::Backend;
use sc_executor::{CallInWasm, NativeExecutor, WasmExecutionMethod, WasmExecutor};
use sc_service::client::{new_in_mem, Client, ClientConfig, LocalCallExecutor};
use sp_api::{ApiRef, ProvideRuntimeApi};
use sp_core::testing::TaskExecutor;
use sp_runtime::generic::BlockId;
use sp_runtime::BuildStorage;
use sp_state_machine::InspectState;
use sc_executor::native_executor_instance;

// Native executor instance.
native_executor_instance!(
    pub Executor,
    node_template_runtime::api::dispatch,
    node_template_runtime::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);

type ClientTempDef = Client<
    Backend<Block>,
    LocalCallExecutor<Backend<Block>, NativeExecutor<Executor>>,
    Block,
    RuntimeApi,
>;

pub struct ClientTemp {
    client: ClientTempDef,
}

impl ClientTemp {
    pub fn new() -> Result<ClientTemp> {
        Ok(ClientTemp {
            client: new_in_mem::<_, Block, _, _>(
                NativeExecutor::<Executor>::new(WasmExecutionMethod::Interpreted, None, 8),
                &gen_chain_spec_thin()
                    .map_err(|_| failure::err_msg("Failed to build temporary chain-spec"))?
                    .build_storage()
                    .map_err(|_| failure::err_msg("Failed to build temporary chain-spec"))?,
                None,
                None,
                Box::new(TaskExecutor::new()),
                ClientConfig::default(),
            )
            .map_err(|_| failure::err_msg("failed to create in-memory client"))?,
        })
    }
    pub fn new_with_genesis(chain_spec: ChainSpec) -> Result<ClientTemp> {
        Ok(ClientTemp {
            client: new_in_mem::<_, Block, _, _>(
                NativeExecutor::<Executor>::new(WasmExecutionMethod::Interpreted, None, 8),
                &chain_spec
                    .build_storage()
                    .map_err(|_| failure::err_msg("Failed to build provided chain-spec"))?,
                None,
                None,
                Box::new(TaskExecutor::new()),
                ClientConfig::default(),
            )
            .map_err(|_| failure::err_msg("failed to create in-memory client"))?,
        })
    }
    pub fn exec_context<T, F: FnOnce() -> Result<Option<T>>>(&self, f: F) -> Result<Option<T>> {
        let mut res = Ok(None);
        self.client
            .state_at(&BlockId::Number(0))
            .map_err(|_| failure::err_msg(""))?
            .inspect_with(|| {
                res = f();
            });

        res
    }
    pub fn runtime_api<'a>(&'a self) -> ApiRef<'a, RuntimeApiImpl<Block, ClientTempDef>> {
        self.client.runtime_api()
    }
}
