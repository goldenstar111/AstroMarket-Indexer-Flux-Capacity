env NEAR_ENV=local near --keyPath ~/.near/localnet/validator_key.json call $1.test.near dispute_market '{"market_id": "'$2'", "winning_outcome": "1", "stake": "10000000000000000000"}' --gas 300000000000000 --accountId test.near