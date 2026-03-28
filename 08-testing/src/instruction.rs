use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum CalculatorInstruction {
    /// 加法
    Add { a: i64, b: i64 },
    /// 减法
    Subtract { a: i64, b: i64 },
    /// 乘法
    Multiply { a: i64, b: i64 },
    /// 除法
    Divide { a: i64, b: i64 },
}
