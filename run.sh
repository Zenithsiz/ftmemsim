#!/bin/env bash

set -e

RUST_FILE_LOG="trace,ftmemsim::classifiers::hemem=debug"
LOG_FILE="latest.log"

PROFILE="dev"
#PROFILE="release"

#TRACE_FILE="resources/traces/bfs.g5.n5.trace"
TRACE_FILE="resources/traces/bfs.g17.n100.t1.trace"
#TRACE_FILE="resources/traces/bc.g18.n100.t1.trace"

rm -rf "$LOG_FILE"

echo "Simulating"
cargo run --profile "$PROFILE" -p ftmemsim        -- --log-file-append --log-file "$LOG_FILE" \
	"$TRACE_FILE"

echo "Creating graphs"
cargo run --profile "$PROFILE" -p ftmemsim-graphs -- --log-file-append --log-file "$LOG_FILE" \
	page-locations "resources/data/page_locations.json" --output "resources/data/page_locations.svg"

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- --log-file-append --log-file "$LOG_FILE" \
	page-migrations "resources/data/page_locations.json" --output "resources/data/page_migrations.svg"

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- --log-file-append --log-file "$LOG_FILE" \
	page-temperature "resources/data/page_accesses.json" --output "resources/data/page_temperature.svg"
