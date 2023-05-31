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
RAM_CAPACITIES="250 375 500 625 750 875 1000 1125 1250 1375 1500 1625 1750 1875 2000 2125 2250 2375 2500 2625 2750 2875 3000 3125 3250 3375 3500 3625 3750 3875 4000 4125 4250 4375 4500"



# Build everything before-hand
cargo build -p ftmemsim -p ftmemsim-graphs

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
	# TODO: Run these in parallel?
	cargo run -q --profile "$PROFILE" -p ftmemsim-graphs -- \
		page-migrations \
		"$output_file" \
		--output "$graph_migrations_file" \
		--output-width  "$GRAPH_OUTPUT_WIDTH" \
		--output-height "$GRAPH_OUTPUT_HEIGHT" \
		&


	pids+="$! "
done

wait $pids
printf "Finished simulating\n"

printf "Generating `page-migrations-hist-multiple` graph for $outputs_files"
graph_migrations_hist_file="graphs/migrations-hist.$GRAPH_OUTPUT_FORMAT"
cargo run -q --profile "$PROFILE" -p ftmemsim-graphs -- \
	page-migrations-hist-multiple \
	$outputs_files \
	--output "$graph_migrations_hist_file" \
	--output-width  "$GRAPH_OUTPUT_WIDTH" \
	--output-height "$GRAPH_OUTPUT_HEIGHT" \
