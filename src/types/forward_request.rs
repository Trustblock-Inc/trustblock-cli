use ethers::{
    abi::Address,
    contract::EthAbiType,
    types::{ transaction::eip712::{ EIP712Domain, TypedData }, Bytes, U256 },
};

use serde::{ Deserialize, Serialize };

use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize, EthAbiType)]
pub struct ForwardRequest {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub gas: U256,
    pub nonce: U256,
    pub data: Bytes,
    #[serde(rename = "validUntilTime")]
    pub valid_until_time: U256,
}

impl ForwardRequest {
    pub fn get_typed_data(self, eip712_domain: &EIP712Domain) -> eyre::Result<TypedData> {
        let eip712_domain_types =
            json!([
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "version",
            "type": "string"
          },
          {
            "name": "chainId",
            "type": "uint256"
          },
          {
            "name": "verifyingContract",
            "type": "address"
          }
        ]);

        let forwarder_types_json =
            json!([
        { "name": "from", "type": "address" },
           { "name": "to", "type": "address" },
           {"name": "value", "type": "uint256" },
           {"name": "gas", "type": "uint256"  },
           { "name": "nonce", "type": "uint256"  },
           { "name": "data", "type": "bytes"  },
           { "name": "validUntilTime", "type": "uint256"  },
         ]);

        let forwarder_request =
            json!( {
          "types": {
            "EIP712Domain": eip712_domain_types,
            "ForwardRequest": forwarder_types_json,
          },
          "primaryType": "ForwardRequest",
          "domain": eip712_domain,
          "message": self
        });

        Ok(serde_json::from_value(forwarder_request)?)
    }
}