use rust_decimal::prelude::*;

pub mod db;
pub mod fuzzing;

/// Main type to deal with money, which is basically a Decimal
type Monetary = Decimal;