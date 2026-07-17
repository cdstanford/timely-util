//!
//! A simple Either type.
//! Has to derive some things in order to implement timely::ExchangeData.
//!

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub enum Either<D1, D2> {
    Left(D1),
    Right(D2),
}
