pub mod lexer;
pub mod parser;
pub mod ast;
pub mod ir;
pub mod profile;
pub mod passes;
pub mod codegen;

#[cfg(feature = "llvm")]
pub mod llvm_codegen;

pub use lexer::{Lexer, Token, Keyword};
pub use parser::Parser;
pub use ast::{Program, Item, Function, Statement, Expression, Type};
pub use ir::{IrModule, IrFunction, IrInstruction, IrType};
pub use profile::Profile;
pub use passes::{
    apply_profile_passes,
    verify_ownership,
    filter_ast_by_profile,
    validate_imports,
    resolve_imports,
    lint_missing_library_imports,
    validate_ir_import_contract,
    validate_function_calls,
};
pub use codegen::ir_to_c;

#[cfg(feature = "llvm")]
pub use llvm_codegen::LlvmCodegen;
