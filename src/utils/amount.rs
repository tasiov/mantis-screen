use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};
use std::ops::{Div, Mul};

pub fn amount_display_to_raw(amount: f64, decimals: i32) -> Decimal {
    Decimal::from_f64(amount)
        .unwrap()
        .mul(Decimal::from(10_i64.pow(decimals as u32)))
        .round_dp_with_strategy(0, RoundingStrategy::ToZero)
}

pub fn amount_raw_to_display(amount: u64, decimals: i32) -> Decimal {
    Decimal::from_u64(amount)
        .unwrap()
        .div(Decimal::from(10_i64.pow(decimals as u32)))
        .round_dp_with_strategy(decimals as u32, RoundingStrategy::ToZero)
}
