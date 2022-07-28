use std::env;
use mongodb::{ Client, options::{ClientOptions, ResolverConfig} };
use reqwest::Error;
use reqwest::StatusCode;
use std::collections::HashMap;
use serde_json::{ Value };
use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize, Debug)]
struct InsertMintedTokenPOSTBody {
    token_ids: Vec<String>,
    contract_id: String,
    owner_id: String,
}

#[derive(Serialize,Deserialize, Debug)]
struct TransferTokenPOSTBody {
    token_ids: Vec<String>,
    contract_id: String,
    old_owner_id: String,
    new_owner_id: String,
}

#[derive(Serialize,Deserialize, Debug)]
struct AddTokenMarketPOSTBody {
    token_id: String,
    nft_contract_id: String,
    owner_id: String,
    approval_id: u64,
    ft_token_id: String,
    price: String,
    started_at: String,
    ended_at: String,
    is_auction: bool,
}

#[derive(Serialize,Deserialize, Debug)]
struct UpdateTokenMarketPOSTBody {
    token_id: String,
    nft_contract_id: String,
    owner_id: String,
    ft_token_id: String,
    price: String,
}

#[derive(Serialize,Deserialize, Debug)]
struct UnlistTokenMarketPOSTBody {
    token_id: String,
    nft_contract_id: String,
    owner_id: String,
}

#[derive(Serialize,Deserialize, Debug)]
struct BidTokenMarketPOSTBody {
    token_id: String,
    nft_contract_id: String,
    bidder_id: String,
    ft_token_id: String,
    price: String,
}

#[derive(Serialize,Deserialize, Debug)]
struct OfferTokenMarketPOSTBody {
    token_id: String,
    nft_contract_id: String,
    buyer_id: String,
    ft_token_id: String,
    price: String,
}

#[derive(Serialize,Deserialize, Debug)]
struct UnofferTokenMarketPOSTBody {
    token_id: String,
    nft_contract_id: String,
    buyer_id: String,
}

#[derive(Serialize,Deserialize, Debug)]
struct ResolveTokenMarketPOSTBody {
    token_id: String,
    nft_contract_id: String,
    owner_id: String,
    ft_token_id: String,
    price: String,
    buyer_id: String,
    is_offer: bool
}


pub async fn db_connect() -> Client {
    // Load the MongoDB connection string from an environment variable:
    let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");

    let client_options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap_or_else(|_| panic!("Could not connect to database"));
    let client = Client::with_options(client_options).unwrap_or_else(|_| panic!("Malformed client options"));
    
    println!("🔗 Connected to database");

    return client;
}

pub async fn insert_minted_token_in_database(
    token_ids: Vec<String>,
    contract_id: String,
    owner_id: String,
    signature_header: String,
    URL: &str,
) -> Result<(), Error> {

    let final_url = format!("{}/insert_tokens", URL);

    let postbody = InsertMintedTokenPOSTBody {
        token_ids: token_ids,
        contract_id: contract_id.clone(),
        owner_id: owner_id.clone(),
    };
    println!("🔗 request for server{}", final_url);
    //make POST request
   	let client = reqwest::Client::new();
   	let res = client
       .post(final_url)
       .header("Signature", signature_header.clone())
       .json(&postbody)
       .send()
       .await?;

    match res.status() {
            StatusCode::OK => println!("Success when inserting minted tokens in database!"),
                                  
            s => println!("Received response status when inserting token for sale in database --> {:?}: {:?} | {:?}", postbody, s, res),
       };
    
    Ok(())
}

pub async fn transfer_token_in_database(
    token_ids: Vec<String>,
    contract_id: String,
    old_owner_id: String,
    new_owner_id: String,
    signature_header: String,
    URL: &str,
) -> Result<(), Error> {

    let final_url = format!("{}/transfer_tokens", URL);

    let postbody = TransferTokenPOSTBody {
        token_ids: token_ids,
        contract_id: contract_id.clone(),
        old_owner_id: old_owner_id.clone(),
        new_owner_id: new_owner_id.clone(),
    };
    println!("🔗 request for server{}", final_url);
    //make POST request
   	let client = reqwest::Client::new();
   	let res = client
       .post(final_url)
       .header("Signature", signature_header.clone())
       .json(&postbody)
       .send()
       .await?;

    match res.status() {
            StatusCode::OK => println!("Success when transferring tokens in database!"),
                                  
            s => println!("Received response status when transferring token for sale in database --> {:?}: {:?} | {:?}", postbody, s, res),
       };
    
    Ok(())
}

pub async fn list_token_market_in_database(
    token_id: String,
    nft_contract_id: String,
    owner_id: String,
    approval_id: u64,
    ft_token_id: String,
    price: String,
    started_at: String,
    ended_at: String,
    is_auction: bool,
    signature_header: String,
    URL: &str,
) -> Result<(), Error> {

    let final_url = format!("{}/list_token", URL);

    let postbody = AddTokenMarketPOSTBody {
        token_id: token_id,
        nft_contract_id: nft_contract_id,
        owner_id: owner_id,
        approval_id: approval_id,
        ft_token_id: ft_token_id,
        price: price,
        started_at: started_at,
        ended_at: ended_at,
        is_auction: is_auction,
    };
    println!("🔗 request for server{}", final_url);
    //make POST request
   	let client = reqwest::Client::new();
   	let res = client
       .post(final_url)
       .header("Signature", signature_header.clone())
       .json(&postbody)
       .send()
       .await?;

    match res.status() {
            StatusCode::OK => println!("Success when listing tokens in database!"),
                                  
            s => println!("Received response status when listing token for sale in database --> {:?}: {:?} | {:?}", postbody, s, res),
       };
    
    Ok(())
}

pub async fn update_token_market_in_database(
    token_id: String,
    nft_contract_id: String,
    owner_id: String,
    ft_token_id: String,
    price: String,
    signature_header: String,
    URL: &str,
) -> Result<(), Error> {

    let final_url = format!("{}/update_token", URL);

    let postbody = UpdateTokenMarketPOSTBody {
        token_id: token_id,
        nft_contract_id: nft_contract_id,
        owner_id: owner_id,
        ft_token_id: ft_token_id,
        price: price,
    };
    println!("🔗 request for server{} ", final_url);
    //make POST request
   	let client = reqwest::Client::new();
   	let res = client
       .post(final_url)
       .header("Signature", signature_header.clone())
       .json(&postbody)
       .send()
       .await?;

    match res.status() {
            StatusCode::OK => println!("Success when updating tokens in database!"),
                                  
            s => println!("Received response status when updating token for sale in database --> {:?}: {:?} | {:?}", postbody, s, res),
       };
    
    Ok(())
}

pub async fn delete_token_market_in_database(
    token_id: String,
    nft_contract_id: String,
    owner_id: String,
    signature_header: String,
    URL: &str,
) -> Result<(), Error> {

    let final_url = format!("{}/unlist_token", URL);

    let postbody = UnlistTokenMarketPOSTBody {
        token_id: token_id,
        nft_contract_id: nft_contract_id,
        owner_id: owner_id,
    };
    println!("🔗 request for server{} ", final_url);
    //make POST request
   	let client = reqwest::Client::new();
   	let res = client
       .post(final_url)
       .header("Signature", signature_header.clone())
       .json(&postbody)
       .send()
       .await?;

    match res.status() {
            StatusCode::OK => println!("Success when deleting tokens in database!"),
                                  
            s => println!("Received response status when deleting token for sale in database --> {:?}: {:?} | {:?}", postbody, s, res),
       };
    
    Ok(())
}

pub async fn bid_token_market_in_database(
    token_id: String,
    nft_contract_id: String,
    bidder_id: String,
    ft_token_id: String,
    price: String,
    signature_header: String,
    URL: &str,
) -> Result<(), Error> {

    let final_url = format!("{}/bid_token", URL);

    let postbody = BidTokenMarketPOSTBody {
        token_id: token_id,
        nft_contract_id: nft_contract_id,
        bidder_id: bidder_id,
        ft_token_id: ft_token_id,
        price: price
    };
    println!("🔗 request for server{} ", final_url);
    //make POST request
   	let client = reqwest::Client::new();
   	let res = client
       .post(final_url)
       .header("Signature", signature_header.clone())
       .json(&postbody)
       .send()
       .await?;

    match res.status() {
            StatusCode::OK => println!("Success when bidding token in database!"),
                                  
            s => println!("Received response status when bidding token for sale in database --> {:?}: {:?} | {:?}", postbody, s, res),
       };
    
    Ok(())
}

pub async fn offer_token_market_in_database(
    token_id: String,
    nft_contract_id: String,
    buyer_id: String,
    ft_token_id: String,
    price: String,
    signature_header: String,
    URL: &str,
) -> Result<(), Error> {

    let final_url = format!("{}/offer_token", URL);

    let postbody = OfferTokenMarketPOSTBody {
        token_id: token_id,
        nft_contract_id: nft_contract_id,
        buyer_id: buyer_id,
        ft_token_id: ft_token_id,
        price: price
    };
    println!("🔗 request for server{} ", final_url);
    //make POST request
   	let client = reqwest::Client::new();
   	let res = client
       .post(final_url)
       .header("Signature", signature_header.clone())
       .json(&postbody)
       .send()
       .await?;

    match res.status() {
            StatusCode::OK => println!("Success when offering token in database!"),
                                  
            s => println!("Received response status when offering token for sale in database --> {:?}: {:?} | {:?}", postbody, s, res),
       };
    
    Ok(())
}

pub async fn unoffer_token_market_in_database(
    token_id: String,
    nft_contract_id: String,
    buyer_id: String,
    signature_header: String,
    URL: &str,
) -> Result<(), Error> {

    let final_url = format!("{}/unoffer_token", URL);

    let postbody = UnofferTokenMarketPOSTBody {
        token_id: token_id,
        nft_contract_id: nft_contract_id,
        buyer_id: buyer_id,
    };
    println!("🔗 request for server{} ", final_url);
    //make POST request
   	let client = reqwest::Client::new();
   	let res = client
       .post(final_url)
       .header("Signature", signature_header.clone())
       .json(&postbody)
       .send()
       .await?;

    match res.status() {
            StatusCode::OK => println!("Success when unoffering token in database!"),
                                  
            s => println!("Received response status when unoffering token for sale in database --> {:?}: {:?} | {:?}", postbody, s, res),
       };
    
    Ok(())
}

pub async fn resolve_token_market_in_database(
    token_id: String,
    nft_contract_id: String,
    owner_id: String,
    ft_token_id: String,
    price: String,
    buyer_id: String,
    is_offer: bool,
    signature_header: String,
    URL: &str,
) -> Result<(), Error> {

    let final_url = format!("{}/resolve_token", URL);

    let postbody = ResolveTokenMarketPOSTBody {
        token_id: token_id,
        nft_contract_id: nft_contract_id,
        owner_id: owner_id,
        ft_token_id: ft_token_id,
        price: price,
        buyer_id: buyer_id,
        is_offer: is_offer
    };
    println!("🔗 request for server{} ", final_url);
    //make POST request
   	let client = reqwest::Client::new();
   	let res = client
       .post(final_url)
       .header("Signature", signature_header.clone())
       .json(&postbody)
       .send()
       .await?;

    match res.status() {
            StatusCode::OK => println!("Success when resolving token in database!"),
                                  
            s => println!("Received response status when resolving token for sale in database --> {:?}: {:?} | {:?}", postbody, s, res),
       };
    
    Ok(())
}