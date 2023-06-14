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

OUTPUT_FILE="resources/data/output.bin.gz"

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
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-migrations-hist \
	"$OUTPUT_FILE" \
	--output "resources/data/page_migrations_hist.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
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
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-temperature-density \
	"$OUTPUT_FILE" \
	--output "resources/data/page_temperature_density.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--temp-exponent "0.15" \
	--temp-read-weight "1.0" \
	--temp-write-weight "2.0" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-temperature-density \
	"$OUTPUT_FILE" \
	--output "resources/data/page_temperature_density.read.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--temp-exponent "0.15" \
	--temp-write-weight "0.0" \
	&

cargo run --profile "$PROFILE" -p ftmemsim-graphs -- \
	--log-file-append \
	--log-file "$LOG_FILE" \
	page-temperature-density \
	"$OUTPUT_FILE" \
	--output "resources/data/page_temperature_density.write.$GRAPH_OUTPUT_FORMAT" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
	--temp-exponent "0.15" \
	--temp-read-weight "0.0" \
	&

wait
