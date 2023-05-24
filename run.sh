#!/bin/env bash

set -e

RUST_FILE_LOG="trace,ftmemsim::classifiers::hemem=debug"
LOG_FILE="latest.log"

#TRACE_FILE="resources/traces/bfs.g5.n5.trace"
TRACE_FILE="resources/traces/bfs.g17.n100.t1.trace"
#TRACE_FILE="resources/traces/bc.g18.n100.t1.trace"

rm -rf "$LOG_FILE"

echo "Simulating"
cargo run --release -p ftmemsim        -- --log-file-append --log-file "$LOG_FILE" \
	"$TRACE_FILE"

echo "Creating graphs"
cargo run --release -p ftmemsim-graphs -- --log-file-append --log-file "$LOG_FILE" \
	page-locations "resources/data/page_locations.json" --output "resources/data/page_locations.svg"

cargo run --release -p ftmemsim-graphs -- --log-file-append --log-file "$LOG_FILE" \
	page-migrations "resources/data/page_locations.json" --output "resources/data/page_migrations.svg"
