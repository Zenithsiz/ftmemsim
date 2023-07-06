#!/bin/env bash

# Prepare the `lackey` parser
# Note: We always compile the `lackey` parser in release, as it might be too slow otherwise
cargo build --manifest-path "../../Cargo.toml" --release --package "parse-lackey"
parse_lackey="../../target/release/parse-lackey"


# Profile to run the example
#PROFILE="dev"
#PROFILE_PATH="debug"
PROFILE="release"
PROFILE_PATH="release"

# Compile the example
cargo build --profile "$PROFILE"

# Then run valgrind on the example and pipe it to the `lackey` parser
../../extern/ftmemsim-valgrind/build/bin/valgrind \
	--tool=ftmemsim_trace \
	"../../target/$PROFILE_PATH/simple-rw" \
	2>&1 \
	1>/dev/null \
	|
	$parse_lackey
