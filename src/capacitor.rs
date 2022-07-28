use near_indexer::near_primitives::{
    views::{
        ExecutionOutcomeWithIdView, 
        ExecutionOutcomeView,
        ExecutionStatusView,
        QueryRequest
    },
    types::{
        BlockReference,
        FunctionArgs
    }
};
use actix::Addr;
use near_client::Query;
use near_client::ViewClientActor;
use tokio_stream::StreamExt;
use mongodb::{ Client, Database, Collection, options::{ UpdateOptions } };
use bson::{ Bson, doc, document::Document };
use serde_json::{ Value };
use serde::{Serialize, Deserialize};
use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8};
use std::vec::Vec;
use std::collections::HashMap;

use crate::database;


/// Metadata for the NFT contract itself.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Mosaics"
    pub symbol: String,            // required, ex. "MOSIAC"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // URL to a JSON file with more info
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

/// Metadata on the individual token level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub media_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
    pub issued_at: Option<String>, // ISO 8601 datetime when token was issued or minted
    pub expires_at: Option<String>, // ISO 8601 datetime when token expires
    pub starts_at: Option<String>, // ISO 8601 datetime when token starts being valid
    pub updated_at: Option<String>, // ISO 8601 datetime when token was last updated
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

/// Note that token IDs for NFTs are strings on NEAR. It's still fine to use autoincrementing numbers as unique IDs if desired, but they should be stringified. This is to make IDs more future-proof as chain-agnostic conventions and standards arise, and allows for more flexibility with considerations like bridging NFTs across chains, etc.
pub type TokenId = String;

/// In this implementation, the Token struct takes two extensions standards (metadata and approval) as optional fields, as they are frequently used in modern NFTs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: Option<TokenMetadata>,
    pub approved_account_ids: Option<HashMap<AccountId, u64>>,
}

pub struct Capacitor {
    capacitor_db: Database,
    database_client: Client,
    allowed_ids: Vec<String>,
}

impl Capacitor {
    pub fn new(database_client: Client, temp_allowed_ids: Vec<String>) -> Self {
        Self {
            capacitor_db: database_client.database("AstroMarket"),
            allowed_ids: temp_allowed_ids,
            database_client,
        }
    }

    pub async fn load(&mut self) {
		let allowed_collection: Collection<Document> = self.capacitor_db.collection("allowed_account_ids");
		let mut cursor = allowed_collection.find(None, None).await.unwrap();
        while let Some(doc) = cursor.next().await {
            let allowed_doc = doc.unwrap();
            let account_id = allowed_doc.get("account_id").and_then(Bson::as_str).unwrap();

            if !self.allowed_ids.contains(&account_id.to_string()) {
                self.allowed_ids.push(account_id.to_string());
            }
        }

        println!("📝 Listening for the following contracts: {:?}", self.allowed_ids);
    }

    pub async fn add_account_id(&mut self, account_id: String) {
        let allowed_collection: Collection<Document> = self.capacitor_db.collection("allowed_account_ids");
        let doc = doc! {
            "account_id": account_id.to_string(),
        };

        let mut cursor = allowed_collection.find(doc.clone(), None).await.unwrap();

        while let Some(doc) = cursor.next().await {
            let allowed_doc = doc.unwrap();
            let doc_account_id = allowed_doc.get("account_id").and_then(Bson::as_str).unwrap();

            if doc_account_id == account_id {
                return ();
            }
        }

        allowed_collection.insert_one(doc.clone(), None).await.unwrap();
        self.allowed_ids.push(account_id.to_string());
    }

    pub fn is_valid_receipt(&self, execution_outcome: &ExecutionOutcomeWithIdView) -> bool {
        match &execution_outcome.outcome.status {
            ExecutionStatusView::SuccessValue(_) => (),
            ExecutionStatusView::SuccessReceiptId(_) => (),
            _ => return false
        }

        self.allowed_ids.contains(&execution_outcome.outcome.executor_id.to_string())
    }

    pub async fn process_outcome(&self, outcome: ExecutionOutcomeView, view_client: Addr<ViewClientActor>, public_api_root: String, signature_header: String) {
        println!("🤖 Processing logs for {}", &outcome.executor_id);
        // let normalized_database_name = "AstroMarket".to_string();
        // let database = self.database_client.database(&normalized_database_name);

        for log in &outcome.logs {
            let logs_str;
            if log.as_str().contains(&"EVENT_JSON:"){
                println!("🤖 Processing logs for 1");
                logs_str = log.as_str().replace("EVENT_JSON:",&"");
                
            } else {
                logs_str = log.to_string();
            }
            let logs_parse = serde_json::from_str(logs_str.as_str());
            
            if logs_parse.is_err() {
                println!("Skipping faulty log: {}", log.as_str());
                continue;
            }

            let logs_parse_value: Option<Value> = logs_parse.unwrap();
            if logs_parse_value.is_some() {
                let parsed_logs: Value = serde_json::from_str(logs_str.as_str()).unwrap();
                let log_type = &parsed_logs["event"].as_str().unwrap_or("None").to_string();
                // let log_type = &parsed_logs["type"].as_str().unwrap_or("None").to_string();
                //colleciton name
                //let collection = database.collection(&"tokens".to_string());
                let contract_id = &outcome.executor_id.as_str().to_string();
                if log_type == "nft_mint" {
                    let params = parsed_logs["data"].as_array().unwrap();
                    for paras in params {
                        let owner_id = &paras["owner_id"].as_str().unwrap().to_string();
                        let tokenvalues = paras["token_ids"].as_array().cloned().unwrap();
                        let tokenids : Vec<String> = tokenvalues.iter().map(|c| c.as_str().unwrap().to_string().clone()).collect();
                        let _put_tokens_status = database::insert_minted_token_in_database(
                            tokenids,
                            contract_id.clone(),
                            owner_id.clone(),
                            signature_header.clone(),
                            &public_api_root
                        ).await;
                    }
                    
			} else if log_type == "nft_transfer" {
                    let params = parsed_logs["data"].as_array().unwrap();
                    for paras in params {
                        let tokenvalues = paras["token_ids"].as_array().cloned().unwrap();
                        let old_owner_id = &paras["old_owner_id"].as_str().unwrap().to_string();
                        let new_owner_id = &paras["new_owner_id"].as_str().unwrap().to_string();
                        let tokenids : Vec<String> = tokenvalues.iter().map(|c| c.as_str().unwrap().to_string().clone()).collect();

                        let _put_tokens_status = database::transfer_token_in_database(
                            tokenids,
                            contract_id.clone(),
                            old_owner_id.clone(),
                            new_owner_id.clone(),
                            signature_header.clone(),
                            &public_api_root
                        ).await;
                    }
                } else if log_type == "add_market_data" {
                    println!("🤖 Processing logs for add_market_data");
                    let params = &parsed_logs["params"];
                    let owner_id = params["owner_id"].as_str().unwrap().to_string();
                    let approval_id = params["approval_id"].as_u64().unwrap();
                    let nft_contract_id = params["nft_contract_id"].as_str().unwrap().to_string();
                    let token_id = params["token_id"].as_str().unwrap_or("None").to_string();
                    let ft_token_id = params["ft_token_id"].as_str().unwrap().to_string();
                    let price = params["price"].as_str().unwrap().to_string();
                    let started_at = params["started_at"].as_str().unwrap_or("0").to_string();
                    let ended_at = params["ended_at"].as_str().unwrap_or("0").to_string();
                    let is_auction = params["is_auction"].as_bool().unwrap_or(false);

                    let _put_tokens_status = database::list_token_market_in_database(
                        token_id,
                        nft_contract_id,
                        owner_id,
                        approval_id,
                        ft_token_id,
                        price,
                        started_at,
                        ended_at,
                        is_auction,
                        signature_header.clone(),
                        &public_api_root
                    ).await;
                } else if log_type == "update_market_data" {
                    println!("🤖 Processing logs for update_market_data");
                    let params = &parsed_logs["params"];
                    let owner_id = params["owner_id"].as_str().unwrap().to_string();
                    let nft_contract_id = params["nft_contract_id"].as_str().unwrap().to_string();
                    let token_id = params["token_id"].as_str().unwrap_or("None").to_string();
                    let ft_token_id = params["ft_token_id"].as_str().unwrap().to_string();
                    let price = params["price"].as_str().unwrap().to_string();

                    let _put_tokens_status = database::update_token_market_in_database(
                        token_id,
                        nft_contract_id,
                        owner_id,
                        ft_token_id,
                        price,
                        signature_header.clone(),
                        &public_api_root
                    ).await;
                } else if log_type == "delete_market_data" {
                    println!("🤖 Processing logs for delete_market_data");
                    let params = &parsed_logs["params"];
                    let owner_id = params["owner_id"].as_str().unwrap().to_string();
                    let nft_contract_id = params["nft_contract_id"].as_str().unwrap().to_string();
                    let token_id = params["token_id"].as_str().unwrap_or("None").to_string();

                    let _put_tokens_status = database::delete_token_market_in_database(
                        token_id,
                        nft_contract_id,
                        owner_id,
                        signature_header.clone(),
                        &public_api_root
                    ).await;
                } else if log_type == "add_bid" {
                    println!("🤖 Processing logs for add_bid");
                    let params = &parsed_logs["params"];
                    let bidder_id = params["bidder_id"].as_str().unwrap().to_string();
                    let nft_contract_id = params["nft_contract_id"].as_str().unwrap().to_string();
                    let token_id = params["token_id"].as_str().unwrap_or("None").to_string();
                    let ft_token_id = params["ft_token_id"].as_str().unwrap().to_string();
                    let price = params["amount"].as_str().unwrap().to_string();

                    let _put_tokens_status = database::bid_token_market_in_database(
                        token_id,
                        nft_contract_id,
                        bidder_id,
                        ft_token_id,
                        price,
                        signature_header.clone(),
                        &public_api_root
                    ).await;
                } else if log_type == "add_offer" {
                    println!("🤖 Processing logs for add_offer");
                    let params = &parsed_logs["params"];
                    let nft_contract_id = params["nft_contract_id"].as_str().unwrap().to_string();
                    let token_id = params["token_id"].as_str().unwrap_or("None").to_string();
                    let buyer_id = params["buyer_id"].as_str().unwrap().to_string();
                    let ft_token_id = params["ft_token_id"].as_str().unwrap().to_string();
                    let price = params["price"].as_str().unwrap().to_string();
                    let _put_tokens_status = database::offer_token_market_in_database(
                        token_id,
                        nft_contract_id,
                        buyer_id,
                        ft_token_id,
                        price,
                        signature_header.clone(),
                        &public_api_root
                    ).await;
                } else if log_type == "delete_offer" {
                    println!("🤖 Processing logs for delete_offer");
                    let params = &parsed_logs["params"];
                    let nft_contract_id = params["nft_contract_id"].as_str().unwrap().to_string();
                    let token_id = params["token_id"].as_str().unwrap_or("None").to_string();
                    let buyer_id = params["buyer_id"].as_str().unwrap().to_string();
                    let _put_tokens_status = database::unoffer_token_market_in_database(
                        token_id,
                        nft_contract_id,
                        buyer_id,
                        signature_header.clone(),
                        &public_api_root
                    ).await;
                } else if log_type == "resolve_purchase" {
                    println!("🤖 Processing logs for resolve_purchase");
                    let params = &parsed_logs["params"];
                    let owner_id = params["owner_id"].as_str().unwrap().to_string();
                    let buyer_id = params["buyer_id"].as_str().unwrap().to_string();
                    let is_offer = params["is_offer"].as_bool().unwrap_or(false);
                    let nft_contract_id = params["nft_contract_id"].as_str().unwrap().to_string();
                    let token_id = params["token_id"].as_str().unwrap_or("None").to_string();
                    let ft_token_id = params["ft_token_id"].as_str().unwrap().to_string();
                    let price = params["price"].as_str().unwrap().to_string();

                    let _put_tokens_status = database::resolve_token_market_in_database(
                        token_id,
                        nft_contract_id,
                        owner_id,
                        ft_token_id,
                        price,
                        buyer_id,
                        is_offer,
                        signature_header.clone(),
                        &public_api_root
                    ).await;
                }
            }
        }
    }
}
