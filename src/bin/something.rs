use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

// extern crate csv;
#[macro_use]



fn main() {
    //           7922816251426433759354395
    let a = dec!(7922816251426433759354395); // 7922816251426433759354395
    let b = dec!(0.0001);
    println!("{}\n{}\n{}", Decimal::MAX, a, a+b);

    let c = dec!(0.01234567);
    println!("{}", c.round_dp(4))

}