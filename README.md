# `ftmemsim`

A tiered-memory simulator.

# Usage

To use this library, first clone it:

```bash
git clone https://github.com/Zenithsiz/ftmemsim
```

(If you need to generate traces, also clone the submodules with `--recurse-submodules`).

After this, you can check `run-tracer.sh` for an example on how to trace a program. You'll need to compile the valgrind fork in `extern/ftmemsim-valgrind`. Follow the instructions on it's README for building.

You'll also need the `parse-valgrind` rust tool. You can use the following to build it:

```bash
cargo build --release --package "parse-valgrind"
```

You can now run the simulator with these traces. Look in the `run.sh` file for examples, but it boils down to running the simulator itself:

```bash
cargo run --release --package ftmemsim -- \
	--config <config-file> \
	<trace-file> \
	--output <output-file>
```

Finally you can use `ftmemsim-graphs` to generate some graphs from it's output. See `./ftmemsim-graphs --help` for a list of all the graphs. You can run, for example, the following:

```bash
cargo run --release --package ftmemsim-graphs -- \
	page-migrations \
	<simulator-output-file> \
	--config <config-file> \
	--output <output-image>
```
