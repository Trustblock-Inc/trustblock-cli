use crate::{
    cmd::utils::{ get_issue_bytes, get_message_data },
    constants::{
        FORWARDER_ADDRESS,
        FORWARDER_DOMAIN_NAME,
        FORWARDER_DOMAIN_VERSION,
        FORWARDER_ENDPOINT,
        FORWARDER_GENERIC_PARAMETERS,
        TB_CORE_ADDRESS,
        TRUSTBLOCK_API_KEY_HEADER,
    },
    types::{ Audit, Chains, ForwardRequest, TrustblockForwarder, WebhookRequest },
    utils::{ apply_dotenv, get_client, WalletSignerMiddleware },
};

use ethers::{
    abi::Contract,
    core::types::Address,
    prelude::k256::ecdsa::SigningKey,
    providers::Middleware,
    signers::{ Signer, Wallet },
    types::{ transaction::eip712::{ EIP712Domain, Eip712 }, Bytes, TransactionRequest, U256 },
    utils::keccak256,
};

use indicatif::{ ProgressBar, ProgressIterator, ProgressStyle };

use std::{ fs::File, path::Path, str::FromStr, sync::Arc };

use reqwest::Client;

use chrono::Utc;

use eyre::eyre;

pub async fn publish_audit_sc(
    audit_data: &Audit,
    project_name: &str,
    report_hash: String,
    private_key: Option<String>,
    auth_token: &String
) -> eyre::Result<()> {
    apply_dotenv()?;

    let chains = &audit_data.chains;
    let issues = audit_data.issues;

    let tb_core_address = std::env
        ::var("TB_CORE_ADDRESS")
        .unwrap_or_else(|_| TB_CORE_ADDRESS.to_string())
        .parse::<Address>()?;

    let forwarder_address = FORWARDER_ADDRESS.parse::<Address>()?;

    let reqwest_client = Client::new();

    println!("CHAINS: {chains:?}\n");

    let tb_core_contract = Contract::load(File::open(Path::new("src/data/TrustblockCore.json"))?)?;

    let publish_audit_function = tb_core_contract.function("publishAudit")?;

    let issue_bytes = get_issue_bytes(issues);

    let mut project_name = project_name.to_owned().into_bytes();

    project_name.resize(28, 0);

    let mut project_bytes = [0; 28];

    project_bytes.copy_from_slice(&project_name);

    let pb = ProgressBar::new(chains.len() as u64);

    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})"
        )?.progress_chars("#>-")
    );

    for chain in chains.iter().progress_with(pb) {
        let rpc_url = chain.get_rpc_url();

        let data = get_message_data(
            publish_audit_function,
            chain,
            &audit_data.contracts,
            project_bytes,
            report_hash.clone(),
            issue_bytes
        )?;

        let (client, chain_id, wallet) = get_client(rpc_url, private_key.clone()).await?;

        let auditor_wallet_address = client.address();

        let forwarder = TrustblockForwarder::new(forwarder_address, client.clone());

        let tx_request = TransactionRequest::new()
            .from(auditor_wallet_address)
            .to(tb_core_address)
            .data(data.clone());

        // Ensures that our SC accepts transaction
        client
            .call(&tx_request.clone().into(), None).await
            .map_err(|err| eyre!("Could not upload an audit: {err}"))?;

        let estimate_gas = client.estimate_gas(&tx_request.clone().into(), None).await?;

        let gas_limit = estimate_gas.saturating_add(
            (estimate_gas / U256::from(5)).saturating_add(U256::from(50_000))
        );

        let batch_nonce = forwarder.get_nonce(auditor_wallet_address).call().await?;

        let webhook_request = prepare_webhook_request(
            client,
            gas_limit,
            batch_nonce,
            data.clone(),
            chain_id,
            chain,
            wallet,
            tb_core_address
        ).await?;

        let forwarder_endpoint = std::env
            ::var("FORWARDER_ENDPOINT")
            .unwrap_or_else(|_| FORWARDER_ENDPOINT.to_string());

        let response = reqwest_client
            .post(forwarder_endpoint)
            .header(TRUSTBLOCK_API_KEY_HEADER, auth_token)
            .json(&webhook_request)
            .send().await?;

        if !response.status().is_success() {
            return Err(
                eyre!(
                    "Could not upload to SC. Response: {:?}\n Body: {:#?} ",
                    response.status(),
                    response.text().await?
                )
            );
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn prepare_webhook_request(
    client: Arc<WalletSignerMiddleware>,
    gas_limit: U256,
    batch_nonce: U256,
    data: Vec<u8>,
    chain_id: U256,
    chain: &Chains,
    wallet: Wallet<SigningKey>,
    tb_core_address: Address
) -> eyre::Result<WebhookRequest> {
    let forwarder_struct = ForwardRequest {
        from: client.address(),
        to: tb_core_address,
        value: (0).into(),
        gas: gas_limit,
        nonce: batch_nonce,
        data: data.clone().into(),
        valid_until_time: (Utc::now().timestamp() + 60 * 60).into(),
    };

    let eip712_domain = EIP712Domain {
        name: Some(FORWARDER_DOMAIN_NAME.to_string()),
        version: Some(FORWARDER_DOMAIN_VERSION.to_string()),
        chain_id: Some(chain_id),
        verifying_contract: Some(FORWARDER_ADDRESS.parse()?),
        salt: None,
    };

    let forwarder_request = forwarder_struct.clone().get_typed_data(&eip712_domain)?;

    let suffix_data = Bytes::from_str("0x")?;

    let request_type_hash = keccak256(format!("ForwardRequest({FORWARDER_GENERIC_PARAMETERS})"));

    let forwarder_typed_domain_separator = forwarder_request.domain_separator()?;

    let signature_bytes = wallet.sign_typed_data(&forwarder_request).await?.to_vec().into();

    let testnet = chain.get_testnets();

    let webhook_request = WebhookRequest {
        request: forwarder_struct.clone(),
        domain_separator: forwarder_typed_domain_separator.into(),
        request_type_hash: request_type_hash.into(),
        suffix_data,
        signature: signature_bytes,
        chain: testnet,
    };

    Ok(webhook_request)
}