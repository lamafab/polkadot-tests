use super::{Block, BlockId, BlockNumber, Header, UncheckedExtrinsic};
use crate::Result;
use codec::Decode;
use sp_core::H256;
use sp_runtime::generic::{Digest, DigestItem};
use sp_runtime::traits::BlakeTwo256;
use std::convert::{TryFrom, TryInto};
use std::mem;
use std::str::FromStr;
use structopt::StructOpt;

macro_rules! from_str {
    ($($name:ident)*) => {
        $(
            impl FromStr for $name {
                type Err = failure::Error;

                fn from_str(val: &str) -> Result<Self> {
                    Ok($name(val.to_string()))
                }
            }
        )*
    };
}

from_str!(
    TxtHash
    TxtBlockNumber
    TxtExtrinsic
);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawBlock(Vec<u8>);

impl FromStr for RawBlock {
    type Err = failure::Error;

    fn from_str(mut val: &str) -> Result<Self> {
        Ok(RawBlock(hex::decode(&mut val.as_bytes())?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxtTestLayout<T> {
    pub name: String,
    #[serde(rename = "type")]
    pub test_ty: String,
    pub description: String,
    pub genesis: String,
    pub data: T,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TxtHash(String);

impl TryFrom<TxtHash> for H256 {
    type Error = failure::Error;

    fn try_from(val: TxtHash) -> Result<Self> {
        Ok(H256::from_slice(&hex::decode(&val.0.replace("0x", ""))?))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TxtBlockNumber(String);

impl TryFrom<TxtBlockNumber> for BlockNumber {
    type Error = failure::Error;

    fn try_from(val: TxtBlockNumber) -> Result<Self> {
        Ok(BlockNumber::from_str_radix(&val.0.replace("0x", ""), 16)?)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxtExtrinsic(String);

impl TryFrom<TxtExtrinsic> for UncheckedExtrinsic {
    type Error = failure::Error;

    fn try_from(mut val: TxtExtrinsic) -> Result<Self> {
        hex::decode(val.0.replace("0x", ""))
            .map_err(|err| failure::Error::from(err))
            .and_then(|mut bytes| {
                UncheckedExtrinsic::decode(&mut bytes.as_slice()).map_err(|err| err.into())
            })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, StructOpt)]
#[serde(rename_all = "camelCase")]
pub struct TxtBlock {
    #[structopt(flatten)]
    pub header: TxtHeader,
    #[structopt(short, long)]
    pub extrinsics: Vec<TxtExtrinsic>,
}

impl TxtBlock {
    // Convert relevant fields into runtime native types.
    pub fn prep(mut self) -> Result<(BlockId, Header, Vec<UncheckedExtrinsic>)> {
        // Convert into runtime types.
        let at = BlockId::Hash(mem::take(&mut self.header.parent_hash).try_into()?);
        let header = mem::take(&mut self.header).try_into()?;
        let extrinsics = mem::take(&mut self.extrinsics)
            .into_iter()
            .map(|e| e.try_into())
            .collect::<Result<Vec<UncheckedExtrinsic>>>()?;

        Ok((at, header, extrinsics))
    }
}

impl TryFrom<TxtBlock> for Block {
    type Error = failure::Error;

    fn try_from(val: TxtBlock) -> Result<Self> {
        Ok(Block {
            header: val.header.try_into()?,
            extrinsics: val
                .extrinsics
                .into_iter()
                .map(|e| e.try_into())
                .collect::<Result<Vec<UncheckedExtrinsic>>>()?,
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, StructOpt)]
#[serde(rename_all = "camelCase")]
pub struct TxtHeader {
    #[structopt(short, long)]
    pub parent_hash: TxtHash,
    #[structopt(short, long)]
    pub number: TxtBlockNumber,
    #[structopt(short, long)]
    pub state_root: TxtHash,
    #[structopt(short, long)]
    pub extrinsics_root: TxtHash,
    #[structopt(flatten)]
    pub digest: TxtDigest,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, StructOpt)]
#[serde(rename_all = "camelCase")]
pub struct TxtDigest {
    pub logs: Vec<String>,
}

impl TryFrom<TxtHeader> for Header {
    type Error = failure::Error;

    fn try_from(val: TxtHeader) -> Result<Self> {
        Ok(Header {
            parent_hash: val.parent_hash.try_into()?,
            number: val.number.try_into()?,
            state_root: val.state_root.try_into()?,
            extrinsics_root: val.extrinsics_root.try_into()?,
            digest: Digest {
                logs: val
                    .digest
                    .logs
                    .iter()
                    .map(|d| hex::decode(d.replace("0x", "")).map_err(|err| err.into()))
                    .collect::<Result<Vec<Vec<u8>>>>()?
                    .iter_mut()
                    .map(|d| DigestItem::decode(&mut d.as_slice()).map_err(|err| err.into()))
                    .collect::<Result<Vec<DigestItem<H256>>>>()?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Convenience trait for tests.
    pub trait ToH256 {
        fn h256(&self) -> H256;
    }

    impl ToH256 for &str {
        fn h256(&self) -> H256 {
            H256::from_slice(&hex::decode(&self.replace("0x", "")).unwrap())
        }
    }

    #[test]
    fn deserialize_to_header() {
        let txt_header: TxtHeader = serde_json::from_slice(r#"
         {
            "parentHash":"0xd380bee22de487a707cbda65dd9d4e2188f736908c42cf390c8919d4f7fc547c",
            "number":"0x01",
            "stateRoot":"0x01045dae0c5d93a84c3dc1f0131126aa6aa1feb26d10f029166fc0c607468968",
            "extrinsicsRoot":"0xa9439bbc818bd95eadb2c5349bef77ee7cc80a282fcceb9670c2c12f939211b4",
            "digest":{
               "logs":[
                  "0x0642414245b50103000000009ddecc0f00000000a8a9c1d717f3904506e333d0ebbf4eed297d50ab9b7c57458b10182f1c84025ef09d3fb5b5f4cb81688939e6363f95aa8d91645fa7b8abc0a6f37812c777c307df51071082d3ff89d4e1b5ad8f5cd3711ada74292c4808237bdf2b076edb280c",
                  "0x05424142450101f66230eb71705213dd10256e3ca5af07492ac420128ecb8bc98f1fcd1f74986d348addbabd4813f0022835b21d720ecadce66a57480d87dfd51d77f3474cb68b"
               ]
            }
         }
        "#.as_bytes()).unwrap();

        let header = Header::try_from(txt_header).unwrap();
        assert_eq!(
            header.parent_hash,
            "0xd380bee22de487a707cbda65dd9d4e2188f736908c42cf390c8919d4f7fc547c".h256()
        );
        assert_eq!(header.number, 1);
        assert_eq!(
            header.state_root,
            "0x01045dae0c5d93a84c3dc1f0131126aa6aa1feb26d10f029166fc0c607468968".h256()
        );
        assert_eq!(
            header.extrinsics_root,
            "0xa9439bbc818bd95eadb2c5349bef77ee7cc80a282fcceb9670c2c12f939211b4".h256()
        );
        assert_eq!(header.digest, Digest {
            logs: vec![
                DigestItem::decode(&mut hex::decode(b"0642414245b50103000000009ddecc0f00000000a8a9c1d717f3904506e333d0ebbf4eed297d50ab9b7c57458b10182f1c84025ef09d3fb5b5f4cb81688939e6363f95aa8d91645fa7b8abc0a6f37812c777c307df51071082d3ff89d4e1b5ad8f5cd3711ada74292c4808237bdf2b076edb280c").unwrap().as_slice()).unwrap(),
                DigestItem::decode(&mut hex::decode(b"05424142450101f66230eb71705213dd10256e3ca5af07492ac420128ecb8bc98f1fcd1f74986d348addbabd4813f0022835b21d720ecadce66a57480d87dfd51d77f3474cb68b").unwrap().as_slice()).unwrap(),
            ]
        });
    }
}
