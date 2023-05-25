#!/bin/env bash

set -e

RUST_FILE_LOG="trace,ftmemsim::classifiers::hemem=debug"
LOG_FILE="latest.log"

PROFILE="dev"
#PROFILE="release"

#TRACE_FILE="resources/traces/bfs.g5.n5.trace"
TRACE_FILE="resources/traces/bfs.g15.n15.trace"
#TRACE_FILE="resources/traces/bfs.g17.n100.t1.trace"
#TRACE_FILE="resources/traces/bc.g18.n100.t1.trace"

CONFIG="config.json"

OUTPUT_WIDTH="1000"
OUTPUT_HEIGHT="1000"

rm -rf "$LOG_FILE"

echo "Simulating"
cargo run --profile "$PROFILE" -p ftmemsim -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	--config "$CONFIG" \
	"$TRACE_FILE"

echo "Creating graphs"
cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-locations \
	"resources/data/page_locations.json" \
	--output "resources/data/page_locations.png" \
	--output-width  "$OUTPUT_WIDTH" \
	--output-height "$OUTPUT_HEIGHT"

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-migrations \
	"resources/data/page_locations.json" \
	--output "resources/data/page_migrations.png" \
	--output-width  "$OUTPUT_WIDTH" \
	--output-height "$OUTPUT_HEIGHT"

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-temperature \
	"resources/data/page_accesses.json" \
	--output "resources/data/page_temperature.png" \
	--output-width  "$OUTPUT_WIDTH" \
	--output-height "$OUTPUT_HEIGHT"
