use solana_program::instruction::Instruction;
use solana_sdk::compute_budget::ComputeBudgetInstruction;

#[derive(Debug, Clone)]
pub struct ComputeBudgetConfig {
    pub micro_lamports: Option<u64>,
    pub units: Option<u32>,
}

pub struct InstructionWithType {
    pub instruction: Instruction,
    pub instruction_type: String,
}

pub fn add_compute_budget(config: &ComputeBudgetConfig) -> Vec<InstructionWithType> {
    let mut instructions = Vec::new();

    // Add compute unit price instruction if specified
    if let Some(micro_lamports) = config.micro_lamports {
        instructions.push(InstructionWithType {
            instruction: ComputeBudgetInstruction::set_compute_unit_price(micro_lamports),
            instruction_type: String::from("SetComputeUnitPrice"),
        });
    }

    // Add compute unit limit instruction if specified
    if let Some(units) = config.units {
        instructions.push(InstructionWithType {
            instruction: ComputeBudgetInstruction::set_compute_unit_limit(units),
            instruction_type: String::from("SetComputeUnitLimit"),
        });
    }

    instructions
}

// Example usage
pub fn example_usage() {
    let config = ComputeBudgetConfig {
        micro_lamports: Some(1000),
        units: Some(200_000),
    };

    let instructions = add_compute_budget(&config);

    for inst_with_type in instructions {
        println!(
            "Instruction type: {}, Instruction: {:?}",
            inst_with_type.instruction_type, inst_with_type.instruction
        );
    }
}
