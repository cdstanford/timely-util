//!
//! A simple Either type.
//! Has to derive some things in order to implement timely::ExchangeData.
//!

use std::fmt::Debug;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Either<D1, D2> {
    Left(D1),
    Right(D2),
}
