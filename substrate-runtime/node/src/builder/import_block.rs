#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blocks {
    blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub header: Header,
    pub extrinsics: Vec<String>,
    pub post_state: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub parent_hash: String,
    pub number: String,
    pub state_root: String,
    pub extrinsics_root: String,
    pub digest: Digest,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Digest {
    pub logs: Vec<String>,
}
