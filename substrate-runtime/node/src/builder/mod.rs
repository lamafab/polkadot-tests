pub mod import_block;

use node_template_runtime::{Signature, Address, BlockNumber, Header, Block, UncheckedExtrinsic, SignedExtra};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestLayout<T> {
    pub name: String,
    #[serde(rename = "type")]
    pub test_ty: String,
    pub description: String,
    pub genesis: String,
    pub data: T,
}
