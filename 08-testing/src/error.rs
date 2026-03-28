use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum CalculatorError {
    #[error("除数不能为零")]
    DivisionByZero,

    #[error("结果溢出")]
    Overflow,
}

impl From<CalculatorError> for ProgramError {
    fn from(e: CalculatorError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
