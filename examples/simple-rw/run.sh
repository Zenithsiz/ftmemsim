#!/bin/env bash

# Prepare the valgrind tool parser
cargo build --manifest-path "../../Cargo.toml" --release --package "parse-valgrind"
parse_valgrind="../../target/release/parse-valgrind"


# Profile to run the example
#PROFILE="dev"
#PROFILE_PATH="debug"
PROFILE="release"
PROFILE_PATH="release"

# Compile the example
cargo build --profile "$PROFILE"

# Then run valgrind on the example and pipe it to the valgrind parser
../../extern/ftmemsim-valgrind/build/bin/valgrind \
	--tool=ftmemsim_trace \
	"../../target/$PROFILE_PATH/simple-rw" \
	2>&1 \
	1>/dev/null \
	|
	$parse_valgrind
