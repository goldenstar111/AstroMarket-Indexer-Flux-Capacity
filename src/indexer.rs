use std::sync::{ Arc, Mutex };
use tokio::sync::mpsc;
use crate::Capacitor;
use actix::Addr;
use near_client::ViewClientActor;

pub async fn handle_blocks_message(capacitor_ins: Arc<Mutex<Capacitor>>, mut stream: mpsc::Receiver<near_indexer::StreamerMessage>, view_client: Addr<ViewClientActor>, public_api_root: String, signature_header: String) {    
    while let Some(streamer_message) = stream.recv().await {
        println!("⛏ Block height {:?}", streamer_message.block.header.height);
        let capacitor_unwrapped = capacitor_ins.lock().unwrap();
        
        for shard in streamer_message.shards {
            for tx_res in shard.receipt_execution_outcomes {
                if !capacitor_unwrapped.is_valid_receipt(&tx_res.execution_outcome) {
                    continue;
                }
    
                capacitor_unwrapped.process_outcome(tx_res.execution_outcome.outcome, view_client.clone(), public_api_root.clone(), signature_header.clone() ).await;
            }

        }
    }
}
