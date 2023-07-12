#!/bin/env bash

set -e

# Trace file to use
#TRACE_FILE="../../resources/traces/bfs.g15.n15.trace"
TRACE_FILE="../../resources/traces/bfs.g17.n100.t1.trace"

# Profile to run under
PROFILE="release"

# Graph output options
GRAPH_OUTPUT_WIDTH=$((4000))
GRAPH_OUTPUT_HEIGHT=$((1000))
GRAPH_OUTPUT_FORMAT="png"

# Ram capacities to test
RAM_CAPACITIES=$(seq -s " " 200 250 4500)


# Build everything before-hand
cargo build --profile "$PROFILE" -p ftmemsim -p ftmemsim-graphs
mkdir -p config output graphs

# Simulate all configs and then generate the graphs
pids=""
outputs_files=""
for ram_capacity in $RAM_CAPACITIES; do
	config_file="config/$ram_capacity.json"
	output_file="output/$ram_capacity.bin.gz"
	graph_migrations_file="graphs/migrations-$ram_capacity.$GRAPH_OUTPUT_FORMAT"

	outputs_files+="$output_file "

	printf "Simulating $config_file\n"

	# Create the config file and change it's page capacity
	cp "base-config.json" "$config_file" \
		&& \

	jq ".hemem.memories[0].page_capacity = $ram_capacity" "$config_file" \
		| sponge "$config_file" \
		&& \

	# Then run the simulation
	cargo run -q --profile "$PROFILE" -p ftmemsim -- \
		--config "$config_file" \
		"$TRACE_FILE" \
		--output "$output_file" \
		&& \

	# Finally run the graphs
	cargo run -q --profile "$PROFILE" -p ftmemsim-graphs -- \
		page-migrations \
		"$output_file" \
		--config "$config_file" \
		--output "$graph_migrations_file" \
		--output-width  "$GRAPH_OUTPUT_WIDTH" \
		--output-height "$GRAPH_OUTPUT_HEIGHT" \
		&


	pids+="$! "
done

wait $pids
printf "Finished simulating\n"

printf "Generating \`page-migrations-hist-multiple\` graph for $outputs_files"
graph_migrations_hist_file="graphs/migrations-hist.$GRAPH_OUTPUT_FORMAT"
cargo run -q --profile "$PROFILE" -p ftmemsim-graphs -- \
	page-migrations-hist-multiple \
	$outputs_files \
	--output "$graph_migrations_hist_file" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
