#!/bin/env bash

set -e

# Trace file to use
#TRACE_FILE="../traces/bfs.g15.n15.trace"
TRACE_FILE="../traces/bfs.g17.n100.t1.trace"

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
for ram_capacity in $RAM_CAPACITIES; do
	config_file="config/$ram_capacity.json"
	output_file="output/$ram_capacity.json"
	log_file="logs/$ram_capacity.json"
	graph_locations_file="graphs/locations-$ram_capacity.$GRAPH_OUTPUT_FORMAT"
	graph_migrations_file="graphs/migrations-$ram_capacity.$GRAPH_OUTPUT_FORMAT"

	printf "Simulating $config_file\n"

	# Create the config file and change it's page capacity
	cp "base-config.json" "$config_file" \
		&& \

	jq ".hemem.memories[0].page_capacity = $ram_capacity" "$config_file" \
		| sponge "$config_file" \
		&& \


	# Note: We Remove the log file since we append in all next steps
	rm -rf "$log_file" \
		&& \

	# Then run the simulation
	cargo run -q --profile "$PROFILE" -p ftmemsim -- \
		--log-file-append \
		--log-file "$log_file" \
		--config "$config_file" \
		"$TRACE_FILE" \
		--output "$output_file" \
		&& \

	# Finally run the graphs
	# TODO: Run these in parallel?
	cargo run -q --profile "$PROFILE" -p ftmemsim-graphs -- \
		--log-file-append \
		--log-file "$log_file" \
		"$output_file" \
		page-locations \
		--output "$graph_locations_file" \
		--output-width  "$GRAPH_OUTPUT_WIDTH" \
		--output-height "$GRAPH_OUTPUT_HEIGHT" \
		&& \

	cargo run -q --profile "$PROFILE" -p ftmemsim-graphs -- \
		--log-file-append \
		--log-file "$log_file" \
		"$output_file" \
		page-migrations \
		--output "$graph_migrations_file" \
		--output-width  "$GRAPH_OUTPUT_WIDTH" \
		--output-height "$GRAPH_OUTPUT_HEIGHT" \
		&

	pids+="$! "
done

wait $pids
printf "Finished simulating\n"
