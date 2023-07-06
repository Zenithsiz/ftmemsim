#!/bin/env bash

set -e

#CMD="date"
CMD="./extern/gapbs/bfs -f resources/graphs/g17.sg -n 100"

extern/ftmemsim-valgrind/build/bin/valgrind \
	--tool=ftmemsim_trace \
	--trace-children=yes \
	--vgdb=no \
	--log-fd=2 \
	$CMD \
	2> >(./target/release/parse-lackey >&2)
