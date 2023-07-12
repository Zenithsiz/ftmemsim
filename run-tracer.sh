#!/bin/env bash

set -e

cargo build --release --package=parse-valgrind

#CMD="date"
#CMD="./extern/gapbs/bfs -f resources/graphs/g17.sg -n 100"
CMD="./extern/gapbs/bfs -g 19 -n 64"

extern/ftmemsim-valgrind/build/bin/valgrind \
	--tool=ftmemsim_trace \
	--trace-children=yes \
	--vgdb=no \
	--log-fd=2 \
	$CMD \
	2> >(./target/release/parse-valgrind >&2)
