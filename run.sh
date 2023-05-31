#!/bin/env bash

set -e

RUST_FILE_LOG="trace,ftmemsim::classifiers::hemem=debug"
LOG_FILE="latest.log"

#PROFILE="dev"
PROFILE="release"

#TRACE_FILE="resources/traces/bfs.g5.n5.trace"
#TRACE_FILE="resources/traces/bfs.g15.n15.trace"
TRACE_FILE="resources/traces/bfs.g17.n100.t1.trace"
#TRACE_FILE="resources/traces/bc.g18.n100.t1.trace"

OUTPUT_FILE="resources/data/output.bin"

CONFIG="config.json"

GRAPH_OUTPUT_WIDTH=$((8000))
GRAPH_OUTPUT_HEIGHT=$((1000))
GRAPH_OUTPUT_FORMAT="png"

rm -rf "$LOG_FILE"

echo "Simulating"
cargo run --profile "$PROFILE" -p ftmemsim -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	--config "$CONFIG" \
	"$TRACE_FILE" \
	--output "$OUTPUT_FILE"

echo "Creating graphs"
cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	"$OUTPUT_FILE" \
	page-migrations \
	--output "resources/data/page_migrations.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT"

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	"$OUTPUT_FILE" \
	page-migrations-hist \
	--output "resources/data/page_migrations_hist.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT"

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	"$OUTPUT_FILE" \
	page-temperature \
	--output "resources/data/page_temperature.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT"
