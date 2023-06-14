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
valgrind \
	--tool=lackey \
	--trace-mem=yes \
	--log-fd=1 \
	"../../target/$PROFILE_PATH/simple-rw" \
	|
	$parse_lackey
