use crate::{types::PNEAR_DECIMALS, *};
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};

const DATA_IMAGE_SVG_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='34' height='34' fill='none'%3E%3Cdefs%3E%3CradialGradient id='a' cx='0' cy='0' r='1' gradientTransform='rotate(59.657 1.184 9.1) scale(31.96023)' gradientUnits='userSpaceOnUse'%3E%3Cstop offset='0%25' stop-color='%23FFBB41'/%3E%3Cstop offset='100%25' stop-color='%23CB0001'/%3E%3C/radialGradient%3E%3CradialGradient id='d' cx='0' cy='0' r='1' gradientTransform='rotate(82.875 6.74 14.776) scale(25.29919 28.35151)' gradientUnits='userSpaceOnUse'%3E%3Cstop offset='0%25' stop-color='%23FFDA76'/%3E%3Cstop offset='55.714%25' stop-color='%23C10000'/%3E%3Cstop offset='100%25' stop-color='%23800101'/%3E%3C/radialGradient%3E%3CclipPath id='b'%3E%3Crect width='33.227' height='34' x='.8' rx='0'/%3E%3C/clipPath%3E%3Cmask id='c'%3E%3Ccircle cx='17' cy='17' r='17' fill='%23FFF'/%3E%3C/mask%3E%3C/defs%3E%3Cg style='mix-blend-mode:passthrough'%3E%3Ccircle cx='17' cy='17' r='17' fill='url(%23a)'/%3E%3Cg clip-path='url(%23b)'%3E%3Cg fill-rule='evenodd' mask='url(%23c)'%3E%3Cpath fill='url(%23d)' d='M18.2 8.5c1.3-.7 3.4-1.3 5.5-.9 3.3.6 11 4.5 10.3 12.8C32.3 28.2 25.5 34 17.2 34c-1.8 0-3.6-.3-5.2-.8 5.2 1 9 .1 11.4-2.9-4.4 1.1-8.3 1-11.5-.2C7.1 28.5-1 21.7 1.5 10.4 4.1 4.3 10.2 0 17.2 0h1.3C12 2.3 7.5 8.2 8.1 15.2c.7 7.6 5.9 10.2 10.7 9.8 3.8-.5 9.4-6.8 5.8-11.8-2.2-3-4.6-3.8-6.2-4-.4 0-.6-.6-.2-.7zm-2.5-.1c.3-1.4 1.4-2.5 2.9-3 1.1-.4 2-1.1 2.4-2.1l.3-.6.1 2.8c0 .7-.5 1.3-1.3 1.5-1.3.4-2.7 1-3.8 1.9l-.9.6.3-1.1zM7 30.6c-3-2.2-5.2-5.4-6.2-9.1 1.8 3.8 5.3 7.2 10.6 10.2-1.4-.1-3-.5-4.4-1.1z'/%3E%3Cpath fill='%23000' fill-opacity='.56' d='M23.2 30.6c-2 2.4-5.1 3.4-9.2 2.9l-2-.3 2 .5c1 .2 2.1.3 3.2.3 8.3 0 15.1-5.8 16.8-13.6.7-8.3-7-12.2-10.3-12.8-2.1-.4-4.2.2-5.5.9-.4.1-.2.7.2.7 1.6.2 4 1 6.2 4 3.6 5-2 11.3-5.8 11.8-4.8.4-10-2.2-10.7-9.8-.6-6.8 3.6-12.6 9.9-15l.5-.2h-1.3c-7 0-13.1 4.3-15.7 10.4C-1 21.7 7.1 28.5 11.9 30.1c3.1 1.2 6.9 1.2 11.1.3l.4-.1-.2.3zM1.7 10.4Q0 17.9 4.1 23.8q1.6 2.2 3.9 4 2 1.5 4 2.2 4.8 1.7 11.4.1l.6-.1-.4.4q-2.6 3.2-7.1 3.4h.7q3 0 5.8-1 2.7-1 5-2.8 2.2-1.8 3.7-4.3t2.1-5.3q.4-5.4-3.6-9.1-1.5-1.5-3.5-2.4Q25 8 23.6 7.8q-2.6-.5-5.3.8l-.1.3.2.1q3.6.4 6.4 4.1 1.2 1.9 1.2 4-.2 1.9-1.4 3.8-1.1 1.7-2.6 2.9-1.7 1.2-3.2 1.3-1.8.2-3.6-.2-1.9-.5-3.4-1.6-3.4-2.7-3.8-8.1-.5-5.2 2.5-9.5Q13.1 2 17.6.2h-.4Q12 .2 7.8 3q-4.1 2.9-6.1 7.4zm14.1-1.9q.3-1 1-1.8.8-.7 1.8-1.1 1.7-.6 2.5-2.1l.1 2q0 .5-.3.8-.3.4-.9.5-2 .6-3.8 1.9l-.5.3.1-.5zm-.2.8.7-.4c1.1-.9 2.5-1.5 3.8-1.9.8-.2 1.3-.8 1.3-1.5l-.1-2.4v-.4l-.2.4-.1.2c-.4 1-1.3 1.7-2.4 2.1-1.5.5-2.6 1.6-2.9 3l-.2.8-.1.3.2-.2zM7 30.6c-2.7-2-4.8-4.8-5.9-8l-.3-1.1.5 1c1.9 3.3 5.1 6.3 9.7 8.9l.4.3-.4-.1c-1.4-.1-2.7-.5-4-1zm3.5.8q-6-3.5-8.8-7.8.6 1.5 1.6 2.9 1.6 2.3 3.8 3.9 1.7.7 3.4 1z'/%3E%3C/g%3E%3Cpath fill='%23FFF' d='m21.4 10.5-3 4.5c-.1.1-.1.3 0 .4.1.1.3.1.4 0l3-2.6h.2v8.4l-.2-.1-9-10.8c-.3-.3-.8-.6-1.2-.6h-.3c-.9 0-1.6.7-1.6 1.6v11.4c0 .9.7 1.6 1.6 1.6.5 0 1-.3 1.3-.8l3-4.5c.1-.1.1-.3 0-.4-.1-.1-.3-.1-.4 0l-3 2.6H12V12.9c0-.1.1 0 .2 0l9 10.8c.3.3.8.6 1.2.6h.3c.9 0 1.6-.7 1.6-1.6V11.3c0-.9-.7-1.6-1.6-1.6-.5 0-1 .3-1.3.8z'/%3E%3Cpath fill='%233A3A3A' fill-rule='evenodd' d='M21 10.2q.7-.9 1.7-.9.8 0 1.4.6.6.6.6 1.4v11.4q0 .8-.6 1.4-.6.6-1.4.6h-.3q-.9 0-1.4-.7l-8.6-10.3v6.8l2.5-2.2q.1-.2.5-.2l.5.2.2.5-.1.5-3 4.5q-.7.9-1.7.9-.8 0-1.4-.6-.6-.6-.6-1.4V11.3q0-.8.6-1.4.6-.6 1.4-.6h.3q.9 0 1.4.7l8.6 10.3v-6.8l-2.5 2.2q-.1.2-.5.2l-.5-.2-.2-.5.1-.5 3-4.5zm.4.3-3 4.5c-.1.1-.1.3 0 .4.1.1.3.1.4 0l2.8-2.4.2-.2h.2v8.4l-.2-.1-.1-.1-.1-.1-8.8-10.6c-.3-.3-.8-.6-1.2-.6h-.3c-.9 0-1.6.7-1.6 1.6v11.4c0 .9.7 1.6 1.6 1.6.5 0 1-.3 1.3-.8l3-4.5c.1-.1.1-.3 0-.4-.1-.1-.3-.1-.4 0L12.4 21l-.2.2H12V12.9c0-.1.1 0 .2 0l.1.1.1.1 8.8 10.6c.3.3.8.6 1.2.6h.3c.9 0 1.6-.7 1.6-1.6V11.3c0-.9-.7-1.6-1.6-1.6-.5 0-1 .3-1.3.8z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E";

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
