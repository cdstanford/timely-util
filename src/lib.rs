// Timely re-exports
// pub use timely::communication::allocator::generic::Generic;
// pub use timely::communication::allocator::thread::Thread;
pub use timely::dataflow::channels::pact::Pipeline;
// pub use timely::dataflow::scopes::child::Child;
pub use timely::dataflow::scope::Scope;
pub use timely::dataflow::stream::Stream;
pub use timely::progress::timestamp::Timestamp;
// pub use timely::worker::Worker;
// pub use timely::Data;

// Stdlib re-exports
pub use std::time::{Duration, SystemTime};

/// Default nanosecond-based time
/// (used for experiment, generators, and perf)
/// TODO: should be a newtype
type TimeNanos = u128;

// Modules
pub mod ec2;
// pub mod experiment;
pub mod generators;
pub mod operators;
pub mod perf;

// Private modules
mod util;
