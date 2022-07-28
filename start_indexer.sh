#!/bin/bash
while ! cargo run --release -- --home-dir ~/.near/mainnet run
do
  sleep 1
  echo "Restarting program..."
done
