use ethers::contract::abigen;

abigen!(
    TrustblockForwarder,
    "src/data/Forwarder.json",
    event_derives(serde::Deserialize, serde::Serialize)
);