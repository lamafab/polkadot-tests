use super::Header;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxtBlock {
    pub block: String,
    pub header: TxtHeader,
    pub extrinsics: Vec<String>,
    pub post_state: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxtHeader {
    pub parent_hash: String,
    pub number: String,
    pub state_root: String,
    pub extrinsics_root: String,
    pub digest: TxtDigest,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxtDigest {
    pub logs: Vec<String>,
}
