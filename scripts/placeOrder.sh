env NEAR_ENV=local near --keyPath ~/.near/localnet/validator_key.json call $1.test.near place_order '{"market_id": "'$2'", "outcome": "3", "shares": "1000", "price": "50", "affiliate_account_id": ""}' --gas 300000000000000 --accountId test.near