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
#TRACE_FILE="examples/simple-rw/output.trace"
#TRACE_FILE="examples/random-rw/output.trace"
#TRACE_FILE="examples/single-page-rw/output.trace"

OUTPUT_FILE="resources/data/output.bin.gz"

CONFIG="config.json"

GRAPH_OUTPUT_WIDTH="4000"
GRAPH_OUTPUT_HEIGHT="2250"
GRAPH_OUTPUT_FORMAT="png"
GRAPH_POINT_SIZE="2.0"
GRAPH_LINE_WIDTH="4.0"

rm -rf "$LOG_FILE"

echo "Simulating"
cargo run --profile "$PROFILE" -p ftmemsim -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	--config "$CONFIG" \
	"$TRACE_FILE" \
	--output "$OUTPUT_FILE"

echo "Creating graphs"
cargo build --profile "$PROFILE" -p ftmemsim-graphs

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-migrations \
	"$OUTPUT_FILE" \
	--config "$CONFIG" \
	--output "resources/data/page_migrations.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--point-size "$GRAPH_POINT_SIZE" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-migrations-hist \
	"$OUTPUT_FILE" \
	--output "resources/data/page_migrations_hist.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--line-width "$GRAPH_LINE_WIDTH" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-location \
	"$OUTPUT_FILE" \
	--config "$CONFIG" \
	--output "resources/data/page_location.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--point-size "$GRAPH_POINT_SIZE" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-temperature \
	"$OUTPUT_FILE" \
	--output "resources/data/page_temperature.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--point-size "$GRAPH_POINT_SIZE" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-temperature-avg \
	"$OUTPUT_FILE" \
	--output "resources/data/page_temperature_avg.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--line-width "$GRAPH_LINE_WIDTH" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	memory-occupancy \
	"$OUTPUT_FILE" \
	--config "$CONFIG" \
	--output "resources/data/memory_occupancy.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--line-width "$GRAPH_LINE_WIDTH" \
	&

wait
