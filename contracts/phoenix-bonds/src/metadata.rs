use crate::*;
use near_sdk::near_bindgen;
use serde::Serialize;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Standard {
    pub standard: String, // standard name e.g. "nep141"
    pub version: String,  // semantic version number of the Standard e.g. "1.0.0"
}

/// To make it easier for the contract to be audited and validated by community
/// and 3rd party, we adopt [NEP-330 standard](https://github.com/near/NEPs/blob/master/neps/nep-0330.md)
/// to make contract source metadata (including versions, source code links and implemented standards)
/// available to auditors, developers and users.
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractSourceMetadata {
    pub version: String,
    pub link: String,
    pub standards: Vec<Standard>,
}

pub trait ContractSourceMetadataTrait {
    fn contract_source_metadata(&self) -> ContractSourceMetadata;
}

#[near_bindgen]
impl ContractSourceMetadataTrait for PhoenixBonds {
    fn contract_source_metadata(&self) -> ContractSourceMetadata {
        ContractSourceMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            link: "https://github.com/linear-protocol/phoenix-bonds".to_string(),
            standards: vec![
                Standard {
                    standard: "nep141".to_string(),
                    version: "1.0.0".to_string(),
                },
                Standard {
                    standard: "nep145".to_string(),
                    version: "1.0.0".to_string(),
                },
                Standard {
                    standard: "nep148".to_string(),
                    version: "1.0.0".to_string(),
                },
                Standard {
                    standard: "nep297".to_string(),
                    version: "1.0.0".to_string(),
                },
                Standard {
                    standard: "nep330".to_string(),
                    version: "1.1.0".to_string(),
                },
            ],
        }
    }
}
