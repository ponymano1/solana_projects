use solana_program::entrypoint;

pub mod instruction;
pub mod processor;
pub mod state;

use processor::process_instruction;

entrypoint!(process_instruction);
