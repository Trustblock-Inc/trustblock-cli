use crate::constants::{ BNB_TESTNET_RPC_URL, FUJI_RPC_URL, GOERLI_RPC_URL, MUMBAI_RPC_URL };

use serde::{ Deserialize, Serialize };

use strum::{ EnumIter, EnumString };

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    Hash,
    Eq,
    PartialEq,
    Serialize,
    Default,
    PartialOrd,
    Ord,
    EnumString
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum Chains {
    #[default]
    Ethereum,
    Polygon,
    BnbChain,
    Avalanche,
}

impl Chains {
    #[must_use]
    pub const fn get_rpc_url(&self) -> &str {
        match self {
            Self::Ethereum => GOERLI_RPC_URL,
            Self::Polygon => MUMBAI_RPC_URL,
            Self::Avalanche => FUJI_RPC_URL,
            Self::BnbChain => BNB_TESTNET_RPC_URL,
        }
    }
    #[must_use]
    pub fn get_testnets(&self) -> String {
        let webhook_url = match self {
            Self::Ethereum => "GOERLI",
            Self::Polygon => "MUMBAI",
            Self::Avalanche => "FUJI",
            Self::BnbChain => "BNBCHAIN_TESTNET",
        };

        webhook_url.to_string()
    }
}