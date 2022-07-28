use actix;
use clap::Parser;
use tokio::sync::mpsc;
use std::convert::TryFrom;
use std::sync::{ Arc, Mutex };
use std::env;
use configs::{ init_logging, Opts, SubCommand };
use dotenv::dotenv;

mod configs;
mod capacitor;
mod http_server;
mod indexer;
mod database;

use capacitor::Capacitor;
use http_server::{ start_http_server };
use indexer::{ handle_blocks_message };
use database::{ db_connect };

use near_indexer;
use actix::Addr;
use near_client::ViewClientActor;

async fn start_process(stream: mpsc::Receiver<near_indexer::StreamerMessage>, view_client: Addr<ViewClientActor>) {
    let public_api = env::var("PUBLIC_API").expect("PUBLIC_API is required to be defined in the .env file");
    let signature = env::var("API_TOKEN").expect("API_TOKEN is required to be defined in the .env file");
    let database_client = db_connect().await;
    let mut capacitor_ins = Capacitor::new(database_client, vec![]);
    capacitor_ins.load().await;

    let mutex_capacitor: Mutex<Capacitor> = Mutex::new(capacitor_ins);
    let wrapped_capacitor = Arc::new(mutex_capacitor);

    actix::spawn(handle_blocks_message(wrapped_capacitor.clone(), stream, view_client, public_api, signature));
    actix::spawn(start_http_server(wrapped_capacitor.clone()));
}
    
fn main() {
    // We use it to automatically search the for root certificates to perform HTTPS calls
    // (sending telemetry and downloading genesis)
    println!("🚀 Starting flux capacitor");
    openssl_probe::init_ssl_cert_env_vars();
    init_logging();
    dotenv().ok();
    
    let opts: Opts = Opts::parse();
    let home_dir = opts.home_dir.unwrap_or(std::path::PathBuf::from(near_indexer::get_default_home()));
    let start_height  = 67779380 as u64;
    
    match opts.subcmd {
        SubCommand::Run => {
            let indexer_config = near_indexer::IndexerConfig {
                home_dir,
                sync_mode: near_indexer::SyncModeEnum::LatestSynced,
                await_for_node_synced: near_indexer::AwaitForNodeSyncedEnum::WaitForFullSync
            };
            let sys = actix::System::new();
            sys.block_on(async move {
                let indexer = near_indexer::Indexer::new(indexer_config).expect("Failed to initiate Indexer");
                let stream = indexer.streamer();
                let view_client = indexer.client_actors().0; //returns tuple, second is another client actor - we only care about first value
                actix::spawn(start_process(stream, view_client));
            });
            sys.run().unwrap();
        }
        SubCommand::Init(config) => near_indexer::init_configs(
            &home_dir,
            config.chain_id.as_ref().map(AsRef::as_ref),
            config.account_id.map(|account_id_string| {
                near_indexer::near_primitives::types::AccountId::try_from(account_id_string)
                    .expect("Received accound_id is not valid")
            }),            config.test_seed.as_ref().map(AsRef::as_ref),
            config.num_shards,
            config.fast,
            config.genesis.as_ref().map(AsRef::as_ref),
            config.download_genesis,
            config.download_genesis_url.as_ref().map(AsRef::as_ref),
            config.download_config,
            config.download_config_url.as_ref().map(AsRef::as_ref),
            config.boot_nodes.as_ref().map(AsRef::as_ref),
            None,
        ).expect("Failed to initiate configs")
    }
}
