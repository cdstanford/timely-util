# timely-util

Utility abstractions on top of (the Rust implementation of) [Timely Dataflow](https://github.com/TimelyDataflow/timely-dataflow).

These utility abstractions are particularly useful for running experiments: that is, setting up a Timely computation (over multiple threads, processes, or EC2 instances), feeds it a bunch of input data, and then eventually collects the output and measures things like the latency and throughput of the process.

Most of these files were originally used to run experiments for the [Flumina](https://github.com/angelhof/flumina) development.
