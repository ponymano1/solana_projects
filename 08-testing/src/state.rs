use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CalculatorResult {
    pub result: i64,
    pub operation_count: u64,
}

impl CalculatorResult {
    pub fn space() -> usize {
        8 + 8 // i64 + u64
    }
}
