use falcon_compiler::{
    apply_profile_passes, filter_ast_by_profile, resolve_imports, validate_imports,
    validate_ir_import_contract, verify_ownership, Lexer, Parser, Profile,
};

fn compile_to_ir(source: &str, profile: Profile) -> Result<falcon_compiler::IrModule, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|error| error.to_string())?;
    let mut parser = Parser::new(tokens);
    let mut ast = parser.parse().map_err(|error| error.to_string())?;

    filter_ast_by_profile(&mut ast, profile);
    let source_dir = std::env::temp_dir();
    let resolved_imports = resolve_imports(&mut ast, &source_dir, profile)?;
    validate_imports(&ast, profile)?;

    let mut ir = falcon_compiler::ir::ast_to_ir(&ast)?;
    ir.imports = resolved_imports;
    validate_ir_import_contract(&ir)?;
    apply_profile_passes(&mut ir, profile)?;
    verify_ownership(&ir)?;
    Ok(ir)
}

#[test]
fn same_source_compiles_across_profiles_with_profile_entrypoints() {
    let source = r#"
#[userland]
func main() {
    let x = 1 + 2;
}

extern func hw_init();

#[kernel]
func kernel_main() {
    loop {}
}

#[baremetal]
func _start() {
    loop {}
}
"#;

    let userland_ir = compile_to_ir(source, Profile::Userland).expect("userland must compile");
    assert!(userland_ir.functions.iter().any(|function| function.name == "main"));

    let kernel_ir = compile_to_ir(source, Profile::Kernel).expect("kernel must compile");
    assert!(kernel_ir
        .functions
        .iter()
        .any(|function| function.name == "kernel_main"));

    let baremetal_ir = compile_to_ir(source, Profile::Baremetal).expect("baremetal must compile");
    assert!(baremetal_ir
        .functions
        .iter()
        .any(|function| function.name == "_start"));
}

#[test]
fn routed_import_is_profile_legal_only_in_userland() {
    let source = r#"
import random;

func main() {
    let x = falcon_randint(1, 10);
}
"#;

    compile_to_ir(source, Profile::Userland).expect("userland routed import should compile");
    let kernel_error =
        compile_to_ir(source, Profile::Kernel).expect_err("kernel routed import must fail");
    assert!(kernel_error.contains("KERNEL PROFILE VIOLATION"));

    let baremetal_error =
        compile_to_ir(source, Profile::Baremetal).expect_err("baremetal routed import must fail");
    assert!(baremetal_error.contains("BAREMETAL PROFILE VIOLATION"));
}

#[test]
fn explicit_import_and_explicit_mod_import_have_same_ir_behavior() {
    let explicit_source = r#"
import random;

func main() {
    let x = falcon_randint(1, 10);
}
"#;

    let explicit_mod_source = r#"
import random::mod;

func main() {
    let x = falcon_randint(1, 10);
}
"#;

    let explicit_ir = compile_to_ir(explicit_source, Profile::Userland).expect("explicit import should compile");
    let explicit_mod_ir =
        compile_to_ir(explicit_mod_source, Profile::Userland).expect("explicit mod import should compile");

    let explicit_main = explicit_ir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main must exist");
    let explicit_mod_main = explicit_mod_ir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main must exist");

    assert_eq!(explicit_main.body.instructions, explicit_mod_main.body.instructions);
}
