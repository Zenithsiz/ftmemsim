#!/bin/env bash

set -e

#PROFILE="dev"
PROFILE="release"

TRACE_FILE="resources/traces/bfs.g19.n64.trace"
#TRACE_FILE="examples/simple-rw/output.trace"
#TRACE_FILE="examples/random-rw/output.trace"
#TRACE_FILE="examples/single-page-rw/output.trace"
#TRACE_FILE="output.trace"

OUTPUT_FILE="resources/data/output.bin.gz"

CONFIG="config.json"

GRAPH_OUTPUT_WIDTH="5000"
GRAPH_OUTPUT_HEIGHT="2500"
GRAPH_OUTPUT_FORMAT="png"
GRAPH_POINT_SIZE="1.0"
GRAPH_LINE_WIDTH="2.0"

echo "Simulating"
cargo run --profile "$PROFILE" -p ftmemsim -- \
	--config "$CONFIG" \
	"$TRACE_FILE" \
	--output "$OUTPUT_FILE"

echo "Creating graphs"
cargo build --profile "$PROFILE" -p ftmemsim-graphs

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	page-migrations \
	"$OUTPUT_FILE" \
	--config "$CONFIG" \
	--output "resources/data/page_migrations.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--point-size "$GRAPH_POINT_SIZE" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	page-migrations-hist \
	"$OUTPUT_FILE" \
	--output "resources/data/page_migrations_hist.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--line-width "$GRAPH_LINE_WIDTH" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	page-location \
	"$OUTPUT_FILE" \
	--config "$CONFIG" \
	--output "resources/data/page_location.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--point-size "$GRAPH_POINT_SIZE" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	page-temperature \
	"$OUTPUT_FILE" \
	--output "resources/data/page_temperature.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--point-size "$GRAPH_POINT_SIZE" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	page-temperature-avg \
	"$OUTPUT_FILE" \
	--output "resources/data/page_temperature_avg.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--line-width "$GRAPH_LINE_WIDTH" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	memory-occupancy \
	"$OUTPUT_FILE" \
	--config "$CONFIG" \
	--output "resources/data/memory_occupancy.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--line-width "$GRAPH_LINE_WIDTH" \
	&

wait
