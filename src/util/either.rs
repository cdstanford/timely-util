use abomonation_derive::Abomonation;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Abomonation)]
pub enum Either<D1, D2> {
    Left(D1),
    Right(D2),
}
