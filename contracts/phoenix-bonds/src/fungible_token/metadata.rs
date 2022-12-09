use crate::{types::PNEAR_DECIMALS, *};
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};

const DATA_IMAGE_SVG_ICON: &str = "";

#[near_bindgen]
impl FungibleTokenMetadataProvider for PhoenixBonds {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: String::from("Phoenix NEAR"),
            symbol: String::from("pNEAR"),
            icon: Some(String::from(DATA_IMAGE_SVG_ICON)),
            reference: None,
            reference_hash: None,
            decimals: PNEAR_DECIMALS,
        }
    }
}
