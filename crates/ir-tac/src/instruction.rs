use evmil::bytecode::Instruction as EvmInstruction;
use primitive_types::U256;
use std::fmt::Write;

use crate::symbol::{Global, Symbol, SymbolTable, Type};

#[derive(PartialEq, Debug)]
pub enum Instruction {
    /// `x = y op z`
    BinaryAssign {
        x: Symbol,
        y: Symbol,
        operator: Operator,
        z: Symbol,
    },

    /// `x = op y`
    UnaryAssign {
        x: Symbol,
        operator: Operator,
        y: Symbol,
    },

    /// `branch target`
    UncoditionalBranch { target: Symbol },

    /// `branch target if condition`
    ConditionalBranch { condition: Symbol, target: Symbol },

    /// `call(label, n)`
    Procedure {
        symbol: Global,
        parameters: Vec<Symbol>,
    },

    /// `x = call(label, n)`
    Function {
        symbol: Global,
        x: Symbol,
        parameters: Vec<Symbol>,
    },

    /// `x = y`
    Copy { x: Symbol, y: Symbol },

    /// `x[index] = y`
    IndexedAssign { x: Symbol, index: Symbol, y: Symbol },

    /// `x = y[index]`
    IndexedCopy { x: Symbol, y: Symbol, index: Symbol },
}

impl Instruction {
    fn target_address(&self) -> Symbol {
        match self {
            Instruction::Copy { x, .. } => *x,
            Instruction::IndexedAssign { x, .. } => *x,
            Instruction::IndexedCopy { x, .. } => *x,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryAssign { x, y, operator, z } => write!(f, "{x} = {y} {operator:?} {z}"),

            Self::UnaryAssign { x, operator, y } => write!(f, "{x} = {operator:?} {y} "),

            Self::UncoditionalBranch { target } => write!(f, "branch {target}"),

            Self::ConditionalBranch { condition, target } => {
                write!(f, "if {condition} branch {target}")
            }

            Self::Procedure { symbol, parameters } => write!(
                f,
                "{symbol:?}({})",
                parameters.iter().fold(String::new(), |mut acc, p| {
                    write!(&mut acc, "{p}, ").unwrap();
                    acc
                })
            ),

            Self::Function {
                symbol,
                x,
                parameters: args,
            } => write!(
                f,
                "{x} = {symbol:?}({})",
                args.iter().fold(String::new(), |mut acc, p| {
                    write!(&mut acc, "{p}, ").unwrap();
                    acc
                })
            ),

            Self::Copy { x, y } => write!(f, "{x} = {y}"),

            Self::IndexedAssign { x, index, y } => write!(f, "{x}[{index}] = {y}"),

            Self::IndexedCopy { x, y, index } => write!(f, "{x} = {y}[{index}]"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Operator {
    Add,
    Mul,
    Sub,
    Div,
    SDiv,
    Mod,
    SMod,
    AddMod,
    MulMod,
    Exp,
    SignExtend,

    LessThat,
    GreaterThan,
    SignedLessThan,
    SignedGreaterThan,
    Eq,
    IsZero,

    And,
    Or,
    Xor,
    Not,
    Byte,
    ShiftLeft,
    ShiftRight,
    ShiftArithmeticRight,
}

struct StackPop {
    decrement: Instruction,
    load: Instruction,
}

/// Pop a value from the stack.
///
/// Returns 2 `Instruction`: Decrementing the stack pointer and the value copy.
fn stack_pop(symbol_table: &mut SymbolTable) -> StackPop {
    let decrement = decrement_stack_height(symbol_table);

    let load = Instruction::IndexedCopy {
        x: symbol_table.temporary(None),
        y: symbol_table.global(Global::Stack),
        index: symbol_table.global(Global::StackHeight),
    };

    StackPop { decrement, load }
}

/// Decrease the stack height by one.
fn decrement_stack_height(symbol_table: &mut SymbolTable) -> Instruction {
    Instruction::BinaryAssign {
        x: symbol_table.global(Global::StackHeight),
        y: symbol_table.global(Global::StackHeight),
        operator: Operator::Sub,
        z: symbol_table.constant(U256::one(), Some(Type::Int(4))),
    }
}

struct StackPush {
    assign: Instruction,
    increment: Instruction,
}

/// Push a `value` to the stack.
///
/// Returns 2 `Instruction`: the value assign and the stack height increase.
fn stack_push(symbol_table: &mut SymbolTable, value: Symbol) -> StackPush {
    let assign = Instruction::IndexedAssign {
        x: symbol_table.global(Global::Stack),
        index: symbol_table.global(Global::StackHeight),
        y: value,
    };
    let increment = increment_stack_height(symbol_table);

    StackPush { assign, increment }
}

/// Increment the stack height by one.
fn increment_stack_height(symbol_table: &mut SymbolTable) -> Instruction {
    Instruction::BinaryAssign {
        x: symbol_table.global(Global::StackHeight),
        y: symbol_table.global(Global::StackHeight),
        operator: Operator::Add,
        z: symbol_table.constant(U256::one(), Some(Type::Int(4))),
    }
}

/// Lower an EVM instruction into corresponding 3AC instructions.
pub fn translate(opcode: &EvmInstruction, symbol_table: &mut SymbolTable) -> Vec<Instruction> {
    use EvmInstruction::*;
    match opcode {
        JUMPDEST => Vec::new(),

        PUSH(bytes) => {
            let type_hint = Some(Type::Bytes(bytes.len()));
            let value = symbol_table.constant(U256::from_big_endian(bytes), type_hint);
            let push = stack_push(symbol_table, value);

            vec![push.assign, push.increment]
        }

        POP => vec![decrement_stack_height(symbol_table)],

        MSTORE => {
            let offset = stack_pop(symbol_table);
            let value = stack_pop(symbol_table);

            let store = Instruction::IndexedAssign {
                x: symbol_table.global(Global::Memory),
                index: offset.load.target_address(),
                y: value.load.target_address(),
            };

            vec![
                offset.decrement,
                offset.load,
                value.decrement,
                value.load,
                store,
            ]
        }

        JUMP => {
            let target = stack_pop(symbol_table);

            let jump = Instruction::UncoditionalBranch {
                target: target.load.target_address(),
            };

            vec![target.decrement, target.load, jump]
        }

        RETURN => {
            let offset = stack_pop(symbol_table);
            let size = stack_pop(symbol_table);

            let procedure = Instruction::Procedure {
                symbol: Global::Return,
                parameters: vec![offset.load.target_address(), size.load.target_address()],
            };

            vec![
                offset.decrement,
                offset.load,
                size.decrement,
                size.load,
                procedure,
            ]
        }

        CALLDATACOPY => {
            let destination_offset = stack_pop(symbol_table);
            let offset = stack_pop(symbol_table);
            let size = stack_pop(symbol_table);

            let parameters = vec![
                destination_offset.load.target_address(),
                offset.load.target_address(),
                size.load.target_address(),
            ];

            let procedure = Instruction::Procedure {
                symbol: Global::MemoryCopy,
                parameters,
            };

            vec![
                destination_offset.decrement,
                destination_offset.load,
                offset.decrement,
                offset.load,
                size.decrement,
                size.load,
                procedure,
            ]
        }

        CALLDATALOAD => {
            let index = stack_pop(symbol_table);

            let value = Instruction::IndexedCopy {
                x: symbol_table.temporary(None),
                y: symbol_table.global(Global::CallData),
                index: index.load.target_address(),
            };

            let push = stack_push(symbol_table, value.target_address());

            vec![
                index.decrement,
                index.load,
                value,
                push.assign,
                push.increment,
            ]
        }

        //_ => todo!("{opcode}"),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use evmil::bytecode;
    use primitive_types::U256;

    use crate::{
        instruction::Operator,
        symbol::{Address, Global, Kind, Symbol, Type},
    };

    use super::Instruction;

    #[test]
    fn lower_push_works() {
        let mut symbol_table = Default::default();

        let opcode = bytecode::Instruction::PUSH(vec![0x01]);
        let result = super::translate(&opcode, &mut symbol_table);

        let expected = vec![
            Instruction::IndexedAssign {
                x: Symbol {
                    address: Address::Label(Global::Stack),
                    type_hint: Type::Word,
                    kind: Global::Stack.kind(),
                },
                index: Symbol {
                    address: Address::Label(Global::StackHeight),
                    type_hint: Type::Int(4),
                    kind: Global::StackHeight.kind(),
                },
                y: Symbol {
                    address: Address::Constant(U256::one()),
                    type_hint: Type::Bytes(1),
                    kind: Kind::Value,
                },
            },
            Instruction::BinaryAssign {
                x: Symbol {
                    address: Address::Label(Global::StackHeight),
                    type_hint: Type::Int(4),
                    kind: Global::StackHeight.kind(),
                },
                y: Symbol {
                    address: Address::Label(Global::StackHeight),
                    type_hint: Type::Int(4),
                    kind: Global::StackHeight.kind(),
                },
                operator: Operator::Add,
                z: Symbol {
                    address: Address::Constant(U256::one()),
                    type_hint: Type::Int(4),
                    kind: Kind::Value,
                },
            },
        ];

        assert_eq!(expected, result);
    }
}
