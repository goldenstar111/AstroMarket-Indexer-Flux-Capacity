🔎 Flux Capacitor
==================================

cargo run -- --home-dir ~/.near/mainnet init --chain-id mainnet --download-genesis

./target/release/indexer-example --home-dir ~/.near/mainnet init --chain-id mainnet --download-genesis --download-config

In order to run copy the `.env.example` to `.env` and run `docker-compose up`

Once started you can call using curl or via browser the following URL to tell Flux Capacitor to watch for logs for a specific contract:

http://localhost:3000/config/add_account?token=YOUR_API_TOKEN&account_id=CONTRACT_ID

Uses the [NEAR Indexer Framework](https://github.com/nearprotocol/nearcore/tree/master/chain/indexer).

Refer to the NEAR Indexer Framework README to learn how to run this example.
