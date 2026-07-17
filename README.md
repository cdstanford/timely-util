# timely-util

Utility abstractions on top of (the Rust implementation of) [Timely Dataflow](https://github.com/TimelyDataflow/timely-dataflow).

These are various abstractions I needed when running experiments on top of Timely and EC2; they include things like monitoring latency and throughput and collecting and saving the output at the end. These files were originally used to run experiments for the [Flumina](https://github.com/angelhof/flumina) development.

### Change log

- `v0.2.0` (2026-07-16): Ported to timely 0.31.0 and Rust 2024.
