/// Profile-specific compiler passes
/// 
/// These passes apply profile-specific transformations and checks to the IR

use crate::ir::{IrInstruction, IrLiteral, IrModule, IrType, IrValue, IntType};
use crate::profile::{Profile, Capability};
use std::collections::{HashMap, HashSet};

/// Filter AST items by profile (compile-time code erasure)
/// 
/// Rules:
/// - Functions with no profile attribute = shared (included in all profiles)
/// - Functions with matching profile attribute = included
/// - Functions with non-matching profile attribute = ERASED
pub fn filter_ast_by_profile(ast: &mut crate::ast::Program, profile: Profile) {
    let profile_name = match profile {
        Profile::Userland => "userland",
        Profile::Kernel => "kernel",
        Profile::Baremetal => "baremetal",
    };
    
    ast.items.retain(|item| {
        match item {
            crate::ast::Item::Function(func) => {
                match &func.profile {
                    None => true,  // Shared function - keep in all profiles
                    Some(p) => p == profile_name,  // Only keep if matches active profile
                }
            }
            // Keep all other items (structs, enums, etc.)
            _ => true,
        }
    });
}

fn profile_name(profile: Profile) -> &'static str {
    match profile {
        Profile::Userland => "userland",
        Profile::Kernel => "kernel",
        Profile::Baremetal => "baremetal",
    }
}

fn is_profile_routed_module(path: &[String]) -> bool {
    matches!(
        path,
        [name]
        if matches!(
            name.as_str(),
            "crypto" | "net" | "ai" | "fs" | "sync" | "time" | "math" | "io" | "random" | "string"
        )
    )
}

fn canonicalize_import_path(path: &[String], profile: Profile) -> Result<Vec<String>, String> {
    if !is_profile_routed_module(path) {
        return Ok(path.to_vec());
    }

    let module = &path[0];
    match profile {
        Profile::Userland => Ok(vec![module.clone(), "mod".to_string()]),
        Profile::Kernel => Err(format!(
            "KERNEL PROFILE VIOLATION:\n\
             Module '{}' resolves to a userland API.\n\
             Use '{}::kernel' or '{}::raw'.",
            module, module, module
        )),
        Profile::Baremetal => Err(format!(
            "BAREMETAL PROFILE VIOLATION:\n\
             Module '{}' requires explicit baremetal-safe import.\n\
             Use '{}::raw'.",
            module, module
        )),
    }
}

fn expand_import_paths(
    import: &crate::ast::Import,
    profile: Profile,
) -> Result<Vec<(Vec<String>, Vec<String>)>, String> {
    if import.selectors.is_empty() {
        return Ok(vec![(
            import.path.clone(),
            canonicalize_import_path(&import.path, profile)?,
        )]);
    }

    let mut expanded = Vec::new();
    for selector in &import.selectors {
        let mut module_path = import.path.clone();
        module_path.push(selector.name.clone());
        let resolved_path = canonicalize_import_path(&module_path, profile)?;
        expanded.push((module_path, resolved_path));
    }
    Ok(expanded)
}

fn capabilities_from_module_path(path: &str) -> Option<HashSet<Capability>> {
    match path {
        "std::net" | "std::net::tcp" | "std::net::udp" | "std::web" => {
            Some([Capability::Heap, Capability::Runtime, Capability::Os].into_iter().collect())
        }
        "std::io" | "std::fs" => {
            Some([Capability::Heap, Capability::Runtime, Capability::Os].into_iter().collect())
        }
        "std::collections" | "std::collections::HashMap" | "std::collections::Vec" => {
            Some([Capability::Heap, Capability::Runtime].into_iter().collect())
        }
        "std::thread" | "std::thread::spawn" => {
            Some(
                [
                    Capability::Heap,
                    Capability::Runtime,
                    Capability::Threads,
                    Capability::Os,
                ]
                .into_iter()
                .collect(),
            )
        }
        "std::sync" | "std::sync::Mutex" | "std::sync::Arc" => {
            Some([Capability::Heap, Capability::Runtime, Capability::Threads].into_iter().collect())
        }
        "core::ptr" | "core::mem" | "core::slice" => {
            Some([Capability::Unsafe].into_iter().collect())
        }
        "ai::llm" | "ai::inference" | "ai::mod" => {
            Some(
                [
                    Capability::Heap,
                    Capability::Runtime,
                    Capability::Threads,
                    Capability::Os,
                ]
                .into_iter()
                .collect(),
            )
        }
        "crypto::mod" => {
            Some([Capability::Heap, Capability::Runtime].into_iter().collect())
        }
        "crypto::kernel" => {
            Some([Capability::Unsafe].into_iter().collect())
        }
        "net::mod" | "fs::mod" => {
            Some([Capability::Heap, Capability::Runtime, Capability::Os].into_iter().collect())
        }
        "sync::mod" => {
            Some([Capability::Threads, Capability::Runtime].into_iter().collect())
        }
        "time::mod" => {
            Some([Capability::Runtime, Capability::Os].into_iter().collect())
        }
        "math::mod" | "random::mod" | "string::mod" => {
            Some([Capability::Runtime].into_iter().collect())
        }
        "io::mod" => {
            Some([Capability::Runtime, Capability::Os].into_iter().collect())
        }
        _ => None,
    }
}

fn get_module_capabilities(path: &[String]) -> HashSet<Capability> {
    if path.last().map(String::as_str) == Some("raw") {
        return [Capability::Unsafe].into_iter().collect();
    }

    for end in (1..=path.len()).rev() {
        let candidate = path[..end].join("::");
        if let Some(required) = capabilities_from_module_path(&candidate) {
            return required;
        }
    }

    HashSet::new()
}

fn is_known_virtual_module(path: &[String]) -> bool {
    !get_module_capabilities(path).is_empty()
        || matches!(path.first().map(String::as_str), Some("std" | "core"))
}

fn ensure_import_path_allowed(path: &[String], profile: Profile) -> Result<(), String> {
    let allowed = profile.allowed_capabilities();
    let required = get_module_capabilities(path);
    let mut forbidden: Vec<_> = required.difference(&allowed).cloned().collect();
    forbidden.sort_by_key(|cap| format!("{:?}", cap));

    if forbidden.is_empty() {
        return Ok(());
    }

    Err(format!(
        "PROFILE VIOLATION: Cannot import '{}' in {} profile.\n\
         Required capabilities: {:?}\n\
         Forbidden capabilities: {:?}\n\
         Hint: Use a profile-safe module path (for example `::raw`) or change profile.",
        path.join("::"),
        profile_name(profile),
        required,
        forbidden
    ))
}

fn is_hosted_io_call(function_name: &str) -> bool {
    matches!(
        function_name,
        "print"
            | "println"
            | "print_int"
            | "print_i32"
            | "print_float"
            | "print_bool"
            | "print_newline"
            | "input"
            | "falcon_print"
            | "falcon_println"
            | "falcon_print_int"
            | "falcon_print_i32"
            | "falcon_print_float"
            | "falcon_print_bool"
            | "falcon_print_newline"
            | "falcon_input"
    )
}

fn capability_path_for_symbol(function_name: &str) -> Option<Vec<String>> {
    let module_hint = required_module_for_symbol(function_name)?;

    let path = if module_hint.contains("::") {
        module_hint
            .split("::")
            .map(|part| part.to_string())
            .collect::<Vec<_>>()
    } else {
        match module_hint {
            // Conservative mapping: std runtime/process helpers require OS/runtime/heap.
            "std" => vec!["std".to_string(), "net".to_string()],
            root => vec![root.to_string(), "mod".to_string()],
        }
    };

    Some(path)
}

fn is_forbidden_runtime_call(function_name: &str, profile: Profile) -> bool {
    let Some(path) = capability_path_for_symbol(function_name) else {
        return false;
    };

    let required = get_module_capabilities(&path);
    if required.is_empty() {
        return false;
    }

    let allowed = profile.allowed_capabilities();
    !required.is_subset(&allowed)
}

fn required_module_for_symbol(function_name: &str) -> Option<&'static str> {
    match function_name {
        "print"
        | "println"
        | "print_int"
        | "print_i32"
        | "print_float"
        | "print_bool"
        | "print_newline"
        | "input"
        | "falcon_print"
        | "falcon_println"
        | "falcon_print_int"
        | "falcon_print_i32"
        | "falcon_print_float"
        | "falcon_print_bool"
        | "falcon_print_newline"
        | "falcon_input"
        | "falcon_str_concat"
        | "falcon_str_eq"
        | "falcon_str_is_empty"
        | "falcon_str_len"
        | "falcon_str_find_from"
        | "falcon_str_substr"
        | "falcon_str_replace_all"
        | "falcon_str_strip_html_tags"
        | "falcon_str_json_extract_values"
        | "falcon_panic" => Some("string"),
        "falcon_random_seed" | "falcon_random" | "falcon_randint" | "falcon_randrange" | "falcon_time_seed" => {
            Some("random")
        }
        "falcon_sin"
        | "falcon_cos"
        | "falcon_tan"
        | "falcon_asin"
        | "falcon_acos"
        | "falcon_atan"
        | "falcon_sqrt"
        | "falcon_pow"
        | "falcon_exp"
        | "falcon_log"
        | "falcon_log10"
        | "falcon_floor"
        | "falcon_ceil"
        | "falcon_round"
        | "falcon_pi"
        | "falcon_e"
        | "falcon_abs"
        | "falcon_min"
        | "falcon_max" => Some("math"),
        "falcon_file_read"
        | "falcon_file_write"
        | "falcon_file_append"
        | "falcon_file_exists"
        | "falcon_file_size" => Some("io"),
        "falcon_ollama_generate"
        | "falcon_ollama_chat"
        | "falcon_ollama_list_models"
            => Some("ai"),
        "read_volatile"
        | "write_volatile" => Some("core::ptr"),
        "falcon_os_exec_capture"
        | "falcon_os_exec_stream" => Some("std"),
        _ => None,
    }
}

fn imported_module_roots(module: &IrModule) -> HashSet<String> {
    module
        .imports
        .iter()
        .filter_map(|import| import.resolved_to.split("::").next())
        .map(|root| root.to_string())
        .collect()
}

/// Ensure freestanding code actually interacts with hardware.
///
/// Baremetal is not "no std" — it is "direct control over hardware."
/// A program that only does pure computation is NOT baremetal; it is
/// misclassified userland code.
///
/// This checks for at least ONE of:
///   - Extern function declarations (hardware boundary)
///   - Volatile load/store (MMIO)
///   - Pointer dereference (register access)
///   - AddrOf operation (taking address of memory)
fn ensure_hardware_reality(module: &IrModule, profile: Profile) -> Result<(), String> {
    // Check 1: extern functions declared (hardware boundary)
    if !module.extern_functions.is_empty() {
        return Ok(());
    }

    // Check 2: scan IR for hardware-related instructions
    for func in &module.functions {
        for inst in &func.body.instructions {
            match inst {
                IrInstruction::VolatileLoad { .. }
                | IrInstruction::VolatileStore { .. }
                | IrInstruction::PtrDeref { .. }
                | IrInstruction::AddrOf { .. } => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    let profile_upper = profile_name(profile).to_uppercase();
    Err(format!(
        "{} PROFILE VIOLATION: No hardware interaction detected.\n\
         {} code must interact with hardware via at least one of:\n\
         - extern func declarations (hardware boundary)\n\
         - volatile reads/writes (MMIO)\n\
         - pointer dereference (register access)\n\
         Pure computation belongs in userland, not {}.",
        profile_upper, profile_upper, profile_name(profile)
    ))
}

fn ensure_entrypoint(
    module: &IrModule,
    profile: Profile,
    accepted: &[&str],
) -> Result<(), String> {
    let entry = module
        .functions
        .iter()
        .find(|function| accepted.iter().any(|name| function.name == *name))
        .ok_or_else(|| {
            format!(
                "{} PROFILE VIOLATION: Missing entrypoint. Expected one of: {}",
                profile_name(profile).to_uppercase(),
                accepted.join(", ")
            )
        })?;

    // Reject explicit returns
    if entry
        .body
        .instructions
        .iter()
        .any(|inst| matches!(inst, IrInstruction::Return { .. }))
    {
        return Err(format!(
            "{} PROFILE VIOLATION: Entrypoint '{}' must not return.",
            profile_name(profile).to_uppercase(),
            entry.name
        ));
    }

    // Verify divergence: entrypoint must end with an infinite loop, halt, or unreachable.
    // Accept: unconditional Branch to a label defined in the function (loop),
    //         Call to "halt" or "unreachable", or empty body (linker-script handles it)
    let diverges = entry.body.instructions.is_empty() || {
        let labels: std::collections::HashSet<&str> = entry.body.instructions.iter()
            .filter_map(|inst| {
                if let IrInstruction::Label { name } = inst { Some(name.as_str()) } else { None }
            })
            .collect();
        entry.body.instructions.iter().rev().any(|inst| match inst {
            IrInstruction::Branch { label } => labels.contains(label.as_str()),
            IrInstruction::Call { func, .. } => func == "halt" || func == "unreachable",
            _ => false,
        })
    };

    if !diverges {
        return Err(format!(
            "{} PROFILE VIOLATION: Entrypoint '{}' must diverge (infinite loop, halt, or unreachable). \
             Pure computation without a control loop is not valid for {}.",
            profile_name(profile).to_uppercase(),
            entry.name,
            profile_name(profile)
        ));
    }

    Ok(())
}

pub fn lint_missing_library_imports(module: &IrModule) -> Result<(), String> {
    let imported_roots = imported_module_roots(module);

    for function in &module.functions {
        for instruction in &function.body.instructions {
            let IrInstruction::Call { func, .. } = instruction else {
                continue;
            };
            let Some(required_root) = required_module_for_symbol(func) else {
                continue;
            };

            if !imported_roots.contains(required_root) {
                return Err(format!(
                    "IMPORT LINT ERROR: Function '{}' in '{}' requires `import {}`.\n\
                     Add an explicit import instead of relying on globally visible runtime symbols.",
                    func, function.name, required_root
                ));
            }
        }
    }

    Ok(())
}

pub fn validate_ir_import_contract(module: &IrModule) -> Result<(), String> {
    let imported_roots = imported_module_roots(module);

    for function in &module.functions {
        for instruction in &function.body.instructions {
            let IrInstruction::Call { func, .. } = instruction else {
                continue;
            };
            let Some(required_root) = required_module_for_symbol(func) else {
                continue;
            };

            if !imported_roots.contains(required_root) {
                return Err(format!(
                    "IR IMPORT CONTRACT VIOLATION: '{}' is called in '{}' without importing '{}'.\n\
                     Backend fallback/inference is forbidden. Resolve imports explicitly in source.",
                    func, function.name, required_root
                ));
            }
        }
    }

    Ok(())
}

/// Validate that all function calls target defined functions
///
/// Checks every Call instruction against:
/// - Regular functions defined in the module
/// - Extern function declarations
/// - Runtime functions (falcon_* prefix — resolved at link time)
pub fn validate_function_calls(module: &IrModule) -> Result<(), String> {
    let mut defined: HashSet<String> = HashSet::new();

    // Collect all defined function names
    for func in &module.functions {
        defined.insert(func.name.clone());
    }

    // Collect extern function declarations
    for name in &module.extern_functions {
        defined.insert(name.clone());
    }

    // Check all calls
    for function in &module.functions {
        check_calls_in_block(&function.body.instructions, &defined, &function.name)?;
    }

    Ok(())
}

fn check_calls_in_block(instructions: &[IrInstruction], defined: &HashSet<String>, caller: &str) -> Result<(), String> {
    for instruction in instructions {
        match instruction {
            IrInstruction::Call { func, .. } => {
                // Allow falcon_* runtime functions (resolved at link time)
                if func.starts_with("falcon_") {
                    continue;
                }
                // Allow well-known runtime functions (short names mapped to falcon_* by codegen)
                if is_known_runtime_function(func) {
                    continue;
                }
                // Allow method calls (Type::method pattern — resolved internally)
                if func.contains("::") {
                    continue;
                }
                if !defined.contains(func) {
                    return Err(format!(
                        "Undefined function '{}' called in '{}'. \
                         Add `extern func {}(...);` declaration or define the function.",
                        func, caller, func
                    ));
                }
            }
            IrInstruction::ClosureCreate { body, .. } => {
                // Check calls inside closure bodies too
                check_calls_in_block(body, defined, caller)?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Known runtime functions that use short names in Falcon source
/// but are mapped to falcon_* symbols by the codegen backends.
fn is_known_runtime_function(name: &str) -> bool {
    matches!(name,
        // I/O
        "println" | "print" | "print_int" | "print_i32" | "print_float" | "print_bool" | "print_newline" | "input" |
        // Math
        "abs" | "min" | "max" | "sqrt" | "pow" | "floor" | "ceil" | "round" |
        // Random
        "rand" | "randrange" | "time_seed" |
        // String
        "strlen" | "strcat" | "strcmp" | "substr" |
        // Type conversion
        "to_string" | "to_int" | "to_float" |
        // System
        "exec" | "sleep" |
        // File I/O
        "file_read" | "file_write" | "file_exists" |
        // Ollama/AI
        "ollama_generate" | "ollama_chat" |
        // Bounds checking (injected by compiler)
        "bounds_check"
    )
}

/// Validate that all imports are compatible with the current profile
/// 
/// This is called during compilation to ensure that imported modules
/// don't require capabilities that the current profile forbids.
pub fn validate_imports(ast: &crate::ast::Program, profile: Profile) -> Result<(), String> {
    for item in &ast.items {
        if let crate::ast::Item::Import(import) = item {
            for (_, resolved_path) in expand_import_paths(import, profile)? {
                ensure_import_path_allowed(&resolved_path, profile)?;
            }
        }
    }
    Ok(())
}

/// Resolve imports by loading and parsing imported files
/// 
/// For each import statement, finds the corresponding .fc file,
/// parses it, and merges its items into the main program.
/// 
/// Import path resolution:
/// - `import foo;` -> looks for `foo.fc` in same directory as source
/// - `import foo::bar;` -> looks for `foo/bar.fc` or `foo.fc` (inner module)
pub fn resolve_imports(
    ast: &mut crate::ast::Program,
    source_dir: &std::path::Path,
    profile: Profile,
) -> Result<Vec<crate::ir::IrImportResolution>, String> {
    let mut imported_items: Vec<crate::ast::Item> = Vec::new();
    let mut resolved_imports: Vec<crate::ir::IrImportResolution> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut active_stack: Vec<String> = Vec::new();
    let top_level_imports: Vec<crate::ast::Import> = ast
        .items
        .iter()
        .filter_map(|item| match item {
            crate::ast::Item::Import(import) => Some(import.clone()),
            _ => None,
        })
        .collect();

    for import in top_level_imports {
        for (module_path, resolved_path) in expand_import_paths(&import, profile)? {
            resolve_import_path_recursive(
                &module_path,
                &resolved_path,
                source_dir,
                profile,
                &mut visited,
                &mut active_stack,
                &mut imported_items,
                &mut resolved_imports,
            )?;
        }
    }
    
    ast.items.retain(|item| !matches!(item, crate::ast::Item::Import(_)));

    let mut new_items = imported_items;
    new_items.append(&mut ast.items);
    ast.items = new_items;
    
    Ok(resolved_imports)
}

fn resolve_import_path_recursive(
    module_path: &[String],
    resolved_path: &[String],
    source_dir: &std::path::Path,
    profile: Profile,
    visited: &mut HashSet<String>,
    active_stack: &mut Vec<String>,
    imported_items: &mut Vec<crate::ast::Item>,
    resolved_imports: &mut Vec<crate::ir::IrImportResolution>,
) -> Result<(), String> {
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    let import_key = resolved_path.join("::");
    if visited.contains(&import_key) {
        return Ok(());
    }

    if let Some(index) = active_stack.iter().position(|entry| entry == &import_key) {
        let mut cycle = active_stack[index..].to_vec();
        cycle.push(import_key.clone());
        return Err(format!("IMPORT CYCLE DETECTED: {}", cycle.join(" -> ")));
    }

    ensure_import_path_allowed(resolved_path, profile)?;

    active_stack.push(import_key.clone());

    let file_path = resolve_import_path(resolved_path, source_dir);
    if file_path.is_none() && is_known_virtual_module(resolved_path) {
        resolved_imports.push(crate::ir::IrImportResolution {
            module: module_path.join("::"),
            resolved_to: resolved_path.join("::"),
            profile: profile_name(profile).to_string(),
        });
        visited.insert(import_key);
        active_stack.pop();
        return Ok(());
    }

    let Some(file_path) = file_path else {
        active_stack.pop();
        return Err(format!(
            "IMPORT RESOLUTION ERROR: Cannot resolve module '{}'.\n\
             No matching source file or registered virtual module was found.",
            resolved_path.join("::")
        ));
    };

    let source = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read imported file '{}': {}", file_path.display(), e))?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer
        .tokenize()
        .map_err(|e| format!("Lexer error in imported file '{}': {}", file_path.display(), e))?;

    let mut parser = Parser::new(tokens);
    let mut imported_ast = parser
        .parse()
        .map_err(|e| format!("Parser error in imported file '{}': {}", file_path.display(), e))?;

    filter_ast_by_profile(&mut imported_ast, profile);

    let nested_source_dir = file_path
        .parent()
        .unwrap_or(source_dir);
    let nested_imports: Vec<crate::ast::Import> = imported_ast
        .items
        .iter()
        .filter_map(|item| match item {
            crate::ast::Item::Import(import) => Some(import.clone()),
            _ => None,
        })
        .collect();

    for nested_import in nested_imports {
        for (nested_module_path, nested_resolved_path) in expand_import_paths(&nested_import, profile)? {
            resolve_import_path_recursive(
                &nested_module_path,
                &nested_resolved_path,
                nested_source_dir,
                profile,
                visited,
                active_stack,
                imported_items,
                resolved_imports,
            )?;
        }
    }

    for imported_item in imported_ast.items {
        if !matches!(imported_item, crate::ast::Item::Import(_)) {
            imported_items.push(imported_item);
        }
    }

    resolved_imports.push(crate::ir::IrImportResolution {
        module: module_path.join("::"),
        resolved_to: resolved_path.join("::"),
        profile: profile_name(profile).to_string(),
    });
    visited.insert(import_key);
    active_stack.pop();
    Ok(())
}

fn resolve_import_path(path: &[String], source_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut roots = vec![source_dir.to_path_buf()];
    if let Some(project_root) = find_project_root(source_dir) {
        roots.push(project_root.join("library"));
        roots.push(project_root.join("stdlib"));
    }

    for root in roots {
        if let Some(path_buf) = resolve_import_from_root(path, &root) {
            return Some(path_buf);
        }
    }

    None
}

fn resolve_import_from_root(path: &[String], root: &std::path::Path) -> Option<std::path::PathBuf> {
    for prefix_len in (1..=path.len()).rev() {
        let prefix = &path[..prefix_len];

        let mut direct = root.to_path_buf();
        for segment in prefix.iter().take(prefix.len().saturating_sub(1)) {
            direct.push(segment);
        }
        if let Some(last) = prefix.last() {
            direct.push(format!("{}.fc", last));
            if direct.exists() {
                return Some(direct);
            }
        }

        let mut module = root.to_path_buf();
        for segment in prefix {
            module.push(segment);
        }
        module.push("mod.fc");
        if module.exists() {
            return Some(module);
        }
    }

    None
}

fn find_project_root(source_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    for ancestor in source_dir.ancestors() {
        if ancestor.join("library").is_dir() && ancestor.join("stdlib").is_dir() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

/// Apply profile-specific passes to IR
pub fn apply_profile_passes(module: &mut IrModule, profile: Profile) -> Result<(), String> {
    // ─── Universal safety analysis (ALL profiles) ───
    warn_null_pointer_usage(module);
    warn_obvious_recursion(module);
    reject_compile_time_oob(module)?;
    warn_pointer_type_mismatch(module);
    warn_empty_delay_loops(module);

    // ─── Profile-specific enforcement ───
    match profile {
        Profile::Userland => {
            // Userland: Verify main function exists, add safety checks
            let has_main = module.functions.iter().any(|f| f.name == "main");
            if !has_main {
                return Err(
                    "USERLAND PROFILE: No 'main' function defined. \
                     Every userland program must have a `func main() { ... }` entry point."
                    .to_string(),
                );
            }
            add_bounds_checks(module)?;
        }
        Profile::Kernel => {
            // Kernel: Enforce restrictions, check for panics, verify no implicit allocations
            enforce_kernel_restrictions(module)?;
        }
        Profile::Baremetal => {
            // Baremetal: Remove all safety checks, optimize for zero overhead
            remove_safety_checks(module)?;
        }
    }
    Ok(())
}
/// ─── B2: Null pointer detection ───
/// Warns when code casts literal 0 to a pointer type (null pointer)
fn warn_null_pointer_usage(module: &IrModule) {
    for func in &module.functions {
        for inst in &func.body.instructions {
            if let IrInstruction::Cast { operand, target_type, .. } = inst {
                let is_zero = matches!(operand, IrValue::Constant(IrLiteral::Int(0)));
                let is_ptr = matches!(target_type, IrType::Pointer { .. });
                if is_zero && is_ptr {
                    eprintln!(
                        "WARNING: Null pointer usage detected in '{}': \
                         casting 0 to pointer type. This will cause a fault on dereference.",
                        func.name
                    );
                }
            }
        }
    }
}

/// ─── B3: Obvious recursion detection ───
/// Warns when a function calls itself — potential infinite recursion / stack overflow
fn warn_obvious_recursion(module: &IrModule) {
    for func in &module.functions {
        let mut calls_self = false;
        let mut has_condition = false;
        for inst in &func.body.instructions {
            match inst {
                IrInstruction::Call { func: callee, .. } => {
                    if callee == &func.name {
                        calls_self = true;
                    }
                }
                IrInstruction::Branch { .. } | IrInstruction::BranchCond { .. } => {
                    has_condition = true;
                }
                _ => {}
            }
        }
        if calls_self && !has_condition {
            eprintln!(
                "WARNING: Function '{}' calls itself with no conditional guard. \
                 This will cause infinite recursion and stack overflow at runtime.",
                func.name
            );
        }
    }
}

/// ─── B5: Compile-time out-of-bounds rejection ───
/// If BOTH the array length AND the index are known constants, reject at compile time
fn reject_compile_time_oob(module: &IrModule) -> Result<(), String> {
    for func in &module.functions {
        let mut known_array_lengths: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for inst in &func.body.instructions {
            match inst {
                IrInstruction::ArrayInit { elements, dest } => {
                    known_array_lengths.insert(dest.name(), elements.len());
                }
                // Track array length through Move/Copy (ArrayInit → Temp → Variable)
                IrInstruction::Move { src, dest } | IrInstruction::Copy { src, dest } => {
                    if let Some(&length) = known_array_lengths.get(&src.name()) {
                        known_array_lengths.insert(dest.name(), length);
                    }
                }
                IrInstruction::Index { base, index, .. } => {
                    if let Some(&array_len) = known_array_lengths.get(&base.name()) {
                        if let IrValue::Constant(IrLiteral::Int(idx)) = index {
                            let idx_val = *idx;
                            if idx_val < 0 || idx_val as usize >= array_len {
                                return Err(format!(
                                    "COMPILE-TIME BOUNDS ERROR in '{}': \
                                     index {} is out of bounds for array of length {} \
                                     (valid range: 0..{})",
                                    func.name, idx_val, array_len, array_len - 1
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// ─── B4: Pointer type mismatch warning ───
/// Warns when casting between pointer types of different widths (*mut u8 → *mut u32)
fn warn_pointer_type_mismatch(module: &IrModule) {
    for func in &module.functions {
        for inst in &func.body.instructions {
            if let IrInstruction::Cast { target_type, dest, .. } = inst {
                // Check if we're casting to a pointer and the dest implies a different pointer type
                // This is a heuristic: if the dest variable name contains type info, check it
                if let IrType::Pointer { inner, .. } = target_type {
                    // Warn for potentially dangerous pointer casts
                    let dest_name = dest.name();
                    if dest_name.contains("u8") && !matches!(**inner, IrType::Int(IntType::U8) | IrType::Int(IntType::I8)) {
                        eprintln!(
                            "WARNING: Possible pointer type mismatch in '{}': \
                             variable '{}' suggests u8 but cast target is different width. \
                             This may cause misaligned access on ARM.",
                            func.name, dest_name
                        );
                    }
                }
            }
        }
    }
}

/// ─── Fix 11: Empty delay loop detection ───
/// Warns about loops that have no function calls, no volatile ops — LLVM may optimize away
fn warn_empty_delay_loops(module: &IrModule) {
    for func in &module.functions {
        // Heuristic: if a function is named "delay" and contains no Call instructions
        // to extern functions (like compiler_fence), warn about optimization
        if func.name.contains("delay") {
            let has_fence_or_call = func.body.instructions.iter().any(|inst| {
                matches!(inst, IrInstruction::Call { func: callee, .. } if callee != &func.name)
            });
            if !has_fence_or_call {
                eprintln!(
                    "WARNING: Function '{}' appears to be a delay loop with no \
                     compiler_fence() or volatile operations. LLVM may optimize it away. \
                     Add `compiler_fence()` inside the loop.",
                    func.name
                );
            }
        }
    }
}

/// Add bounds checking for userland profile
fn add_bounds_checks(module: &mut IrModule) -> Result<(), String> {
    // In userland profile, we add bounds checking for array/vector accesses
    // This is done by wrapping Index instructions with bounds checks
    for func in &mut module.functions {
        let mut new_instructions = Vec::new();
        let mut known_array_lengths: HashMap<String, i64> = HashMap::new();
        let mut bounds_check_counter = 0;
        
        for inst in &func.body.instructions {
            match inst {
                IrInstruction::ArrayInit { elements, dest } => {
                    known_array_lengths.insert(dest.name(), elements.len() as i64);
                    new_instructions.push(inst.clone());
                }
                IrInstruction::Move { src, dest } | IrInstruction::Copy { src, dest } => {
                    let src_name = src.name();
                    let dest_name = dest.name();
                    if let Some(length) = known_array_lengths.get(&src_name).copied() {
                        known_array_lengths.insert(dest_name, length);
                    } else {
                        known_array_lengths.remove(&dest_name);
                    }
                    new_instructions.push(inst.clone());
                }
                IrInstruction::Index { base, index, dest: _ } => {
                    // Insert bounds check before the index operation
                    // The bounds check compares index against a known length when available.
                    // Unknown length is encoded as -1 (runtime still enforces lower bound).
                    let fail_label = format!("bounds_fail_{}", bounds_check_counter);
                    bounds_check_counter += 1;

                    let bound_value = known_array_lengths
                        .get(&base.name())
                        .copied()
                        .unwrap_or(-1);
                    
                    new_instructions.push(IrInstruction::BoundsCheck {
                        value: index.clone(),
                        bound: IrValue::Constant(IrLiteral::Int(bound_value)),
                        on_fail: fail_label,
                    });
                    
                    new_instructions.push(inst.clone());
                }
                _ => {
                    new_instructions.push(inst.clone());
                }
            }
        }
        func.body.instructions = new_instructions;
    }
    Ok(())
}

/// Enforce kernel profile restrictions
/// 
/// This is compile-time law. Invalid programs MUST be rejected here.
fn enforce_kernel_restrictions(module: &mut IrModule) -> Result<(), String> {
    ensure_entrypoint(module, Profile::Kernel, &["_start", "kernel_main"])?;

    for func in &mut module.functions {
        // Check for heap allocations
        for inst in &func.body.instructions {
            match inst {
                IrInstruction::Alloc { .. } | IrInstruction::HeapAlloc { .. } => {
                    // Kernel profile: Heap allocation FORBIDDEN
                    return Err(format!(
                        "KERNEL PROFILE VIOLATION: Heap allocation forbidden in function '{}'. \
                        Kernel profile does not allow heap allocations. Use stack allocation or unsafe custom allocator.",
                        func.name
                    ));
                }
                IrInstruction::Panic { message } => {
                    // Kernel profile: Panics FORBIDDEN
                    return Err(format!(
                        "KERNEL PROFILE VIOLATION: panic!() call forbidden in function '{}'. \
                        Kernel profile does not allow panics. Use Result<T, E> for error handling. \
                        Panic message: {}",
                        func.name, message
                    ));
                }
                IrInstruction::Unwrap { .. } => {
                    // Kernel profile: unwrap() FORBIDDEN (uses panic)
                    return Err(format!(
                        "KERNEL PROFILE VIOLATION: unwrap()/expect() forbidden in function '{}'. \
                        Kernel profile does not allow panics. Use match or ? operator to handle Result/Option.",
                        func.name
                    ));
                }
                IrInstruction::Call { func: called, .. }
                    if is_forbidden_runtime_call(called, Profile::Kernel) =>
                {
                    let detail = if is_hosted_io_call(called) {
                        "Kernel profile has no stdout/stdin runtime. Use explicit device/driver APIs."
                    } else {
                        "Kernel profile forbids hosted runtime/OS helper calls. Use freestanding device/driver bindings."
                    };
                    return Err(format!(
                        "KERNEL PROFILE VIOLATION: runtime call '{}' forbidden in function '{}'. {}",
                        called, func.name, detail
                    ));
                }
                _ => {}
            }
        }
    }

    // Hardware reality check: kernel code must interact with hardware
    ensure_hardware_reality(module, Profile::Kernel)?;

    Ok(())
}

/// Enforce baremetal profile restrictions
/// 
/// This is compile-time law. Invalid programs MUST be rejected here.
fn remove_safety_checks(module: &mut IrModule) -> Result<(), String> {
    ensure_entrypoint(module, Profile::Baremetal, &["_start"])?;

    for func in &mut module.functions {
        // Check for forbidden operations
        for inst in &func.body.instructions {
            match inst {
                IrInstruction::Alloc { .. } | IrInstruction::HeapAlloc { .. } => {
                    // Baremetal: Heap allocation FORBIDDEN
                    return Err(format!(
                        "BAREMETAL PROFILE VIOLATION: Heap allocation forbidden in function '{}'. \
                        Baremetal profile has zero runtime and no allocator.",
                        func.name
                    ));
                }
                IrInstruction::Panic { message } => {
                    // Baremetal: Panics FORBIDDEN (no unwinding support)
                    return Err(format!(
                        "BAREMETAL PROFILE VIOLATION: panic!() call forbidden in function '{}'. \
                        Baremetal profile has no unwinding support. Use custom panic handler or abort. \
                        Panic message: {}",
                        func.name, message
                    ));
                }
                IrInstruction::Unwrap { .. } => {
                    // Baremetal: unwrap() FORBIDDEN
                    return Err(format!(
                        "BAREMETAL PROFILE VIOLATION: unwrap()/expect() forbidden in function '{}'. \
                        Baremetal profile does not allow panics.",
                        func.name
                    ));
                }
                IrInstruction::BoundsCheck { .. } => {
                    // Baremetal: Bounds checks removed (but instruction should not exist)
                    // This is handled in codegen, but we verify here
                }
                IrInstruction::Call { func: called, .. }
                    if is_forbidden_runtime_call(called, Profile::Baremetal) =>
                {
                    let detail = if is_hosted_io_call(called) {
                        "Baremetal is freestanding and has no stdout/stdin runtime."
                    } else {
                        "Baremetal forbids hosted runtime/OS helper calls. Use direct MMIO/board bindings."
                    };
                    return Err(format!(
                        "BAREMETAL PROFILE VIOLATION: runtime call '{}' forbidden in function '{}'. {}",
                        called, func.name, detail
                    ));
                }
                _ => {}
            }
        }
    }

    // Hardware reality check: baremetal code must interact with hardware
    ensure_hardware_reality(module, Profile::Baremetal)?;

    Ok(())
}

/// Ownership verification pass
/// 
/// Verifies that ownership rules are followed:
/// - No use-after-move
/// - No simultaneous mutable and immutable borrows
/// - All borrows outlive their data
pub fn verify_ownership(module: &IrModule) -> Result<(), String> {
    for func in &module.functions {
        let mut ownership_state = OwnershipState::new();
        
        for inst in &func.body.instructions {
            // Phase 1: Check all operands for use-after-move BEFORE processing the instruction
            check_operands_not_moved(inst, &ownership_state, &func.name)?;
            
            match inst {
                IrInstruction::Move { src, dest } => {
                    let src_name = src.name();
                    let dest_name = dest.name();
                    
                    // Reassignment resets ownership: if dest was moved, un-move it
                    ownership_state.moved.remove(&dest_name);
                    ownership_state.owners.insert(dest_name, true);
                    
                    // Only invalidate source for non-Copy types
                    // Copy types: temporaries holding literals, and known scalar variables
                    if !ownership_state.is_copy_value(&src_name) {
                        ownership_state.move_value(src.clone(), dest.clone())?;
                    }
                }
                IrInstruction::BorrowImm { src, dest, lifetime } => {
                    ownership_state.borrow_imm(src.clone(), dest.clone(), lifetime.clone())?;
                }
                IrInstruction::BorrowMut { src, dest, lifetime } => {
                    ownership_state.borrow_mut(src.clone(), dest.clone(), lifetime.clone())?;
                }
                IrInstruction::Drop { value } => {
                    ownership_state.drop_value(value.clone())?;
                }
                IrInstruction::Literal { dest, value } => {
                    // Mark scalar literals as Copy
                    let dest_name = dest.name();
                    ownership_state.owners.insert(dest_name.clone(), true);
                    match value {
                        IrLiteral::Int(_) | IrLiteral::Float(_) | IrLiteral::Bool(_) | IrLiteral::Char(_) => {
                            ownership_state.copy_values.insert(dest_name);
                        }
                        _ => {}
                    }
                }
                IrInstruction::Call { dest, .. } | IrInstruction::ClosureCall { dest, .. } => {
                    // Call result is a new owner
                    if let Some(d) = dest {
                        let dest_name = d.name();
                        ownership_state.moved.remove(&dest_name);
                        ownership_state.owners.insert(dest_name, true);
                    }
                }
                _ => {
                    // Other instructions: destinations become owners
                }
            }
        }
    }
    Ok(())
}

/// Check that no operand of this instruction has been moved
fn check_operands_not_moved(
    inst: &IrInstruction,
    state: &OwnershipState,
    func_name: &str,
) -> Result<(), String> {
    let operands = get_instruction_operands(inst);
    for op in operands {
        let name = op.name();
        if state.moved.contains(&name) && !state.is_copy_value(&name) {
            return Err(format!(
                "Use of moved value '{}' in function '{}'. \
                 The value was moved earlier and can no longer be used.",
                name, func_name
            ));
        }
    }
    Ok(())
}

/// Extract all operand values referenced by an instruction (read operands only)
fn get_instruction_operands(inst: &IrInstruction) -> Vec<&IrValue> {
    match inst {
        IrInstruction::Add { left, right, .. }
        | IrInstruction::Sub { left, right, .. }
        | IrInstruction::Mul { left, right, .. }
        | IrInstruction::Div { left, right, .. }
        | IrInstruction::Mod { left, right, .. }
        | IrInstruction::Eq { left, right, .. }
        | IrInstruction::Ne { left, right, .. }
        | IrInstruction::Lt { left, right, .. }
        | IrInstruction::Le { left, right, .. }
        | IrInstruction::Gt { left, right, .. }
        | IrInstruction::Ge { left, right, .. }
        | IrInstruction::And { left, right, .. }
        | IrInstruction::Or { left, right, .. }
        | IrInstruction::BitAnd { left, right, .. }
        | IrInstruction::BitOr { left, right, .. }
        | IrInstruction::BitXor { left, right, .. }
        | IrInstruction::Shl { left, right, .. }
        | IrInstruction::Shr { left, right, .. } => vec![left, right],
        IrInstruction::Neg { operand, .. }
        | IrInstruction::Not { operand, .. }
        | IrInstruction::BitNot { operand, .. }
        | IrInstruction::AddrOf { operand, .. }
        | IrInstruction::PtrDeref { operand, .. } => vec![operand],
        IrInstruction::Move { src, .. } => vec![src],
        IrInstruction::Call { args, .. } => args.iter().collect(),
        IrInstruction::ClosureCall { closure, args, .. } => {
            let mut ops = vec![closure];
            ops.extend(args.iter());
            ops
        }
        IrInstruction::Return { value: Some(v) } => vec![v],
        IrInstruction::Index { base, index, .. } => vec![base, index],
        IrInstruction::BoundsCheck { value, bound, .. } => vec![value, bound],
        IrInstruction::FieldAccess { base, .. } => vec![base],
        IrInstruction::BorrowImm { src, .. } | IrInstruction::BorrowMut { src, .. } => vec![src],
        IrInstruction::Drop { value } => vec![value],
        IrInstruction::BranchCond { condition, .. } => vec![condition],
        IrInstruction::EnumInit { payload: Some(p), .. } => vec![p],
        IrInstruction::EnumTag { value, .. } => vec![value],
        IrInstruction::EnumPayload { value, .. } => vec![value],
        IrInstruction::Cast { operand, .. } => vec![operand],
        IrInstruction::VolatileStore { value, .. } => vec![value],
        _ => vec![],
    }
}

/// Tracks ownership state during verification
struct OwnershipState {
    owners: std::collections::HashMap<String, bool>, // value -> is_owned
    imm_borrows: std::collections::HashMap<String, Vec<String>>, // value -> [borrow names]
    mut_borrows: std::collections::HashMap<String, Option<String>>, // value -> borrow name (if any)
    moved: std::collections::HashSet<String>, // values that have been moved
    copy_values: std::collections::HashSet<String>, // values that are Copy (scalars)
}

impl OwnershipState {
    fn new() -> Self {
        Self {
            owners: std::collections::HashMap::new(),
            imm_borrows: std::collections::HashMap::new(),
            mut_borrows: std::collections::HashMap::new(),
            moved: std::collections::HashSet::new(),
            copy_values: std::collections::HashSet::new(),
        }
    }
    
    /// Check if a value is a Copy type (scalars don't get invalidated on move)
    fn is_copy_value(&self, name: &str) -> bool {
        self.copy_values.contains(name)
    }
    
    fn move_value(&mut self, src: crate::ir::IrValue, dest: crate::ir::IrValue) -> Result<(), String> {
        let src_name = self.value_name(&src);
        let dest_name = self.value_name(&dest);
        
        // Check: src should be owned and not moved
        if self.moved.contains(&src_name) {
            return Err(format!("Use after move: {}", src_name));
        }
        
        if let Some(borrows) = self.imm_borrows.get(&src_name) {
            if !borrows.is_empty() {
                return Err(format!("Cannot move {}: has active immutable borrows", src_name));
            }
        }
        
        if let Some(Some(_)) = self.mut_borrows.get(&src_name) {
            return Err(format!("Cannot move {}: has active mutable borrow", src_name));
        }
        
        // Mark src as moved
        self.moved.insert(src_name.clone());
        self.owners.remove(&src_name);
        
        // Mark dest as owner
        self.owners.insert(dest_name, true);
        
        Ok(())
    }
    
    fn borrow_imm(&mut self, src: crate::ir::IrValue, dest: crate::ir::IrValue, _lifetime: String) -> Result<(), String> {
        let src_name = self.value_name(&src);
        let dest_name = self.value_name(&dest);
        
        // Check: src should not have active mutable borrow
        if let Some(Some(_)) = self.mut_borrows.get(&src_name) {
            return Err(format!("Cannot immutably borrow {}: has active mutable borrow", src_name));
        }
        
        // Check: src should not be moved
        if self.moved.contains(&src_name) {
            return Err(format!("Cannot borrow {}: has been moved", src_name));
        }
        
        // Add immutable borrow
        self.imm_borrows.entry(src_name).or_insert_with(Vec::new).push(dest_name);
        
        Ok(())
    }
    
    fn borrow_mut(&mut self, src: crate::ir::IrValue, dest: crate::ir::IrValue, _lifetime: String) -> Result<(), String> {
        let src_name = self.value_name(&src);
        let dest_name = self.value_name(&dest);
        
        // Check: src should not have active immutable borrows
        if let Some(borrows) = self.imm_borrows.get(&src_name) {
            if !borrows.is_empty() {
                return Err(format!("Cannot mutably borrow {}: has active immutable borrows", src_name));
            }
        }
        
        // Check: src should not have active mutable borrow
        if let Some(Some(_)) = self.mut_borrows.get(&src_name) {
            return Err(format!("Cannot mutably borrow {}: already mutably borrowed", src_name));
        }
        
        // Check: src should not be moved
        if self.moved.contains(&src_name) {
            return Err(format!("Cannot borrow {}: has been moved", src_name));
        }
        
        // Add mutable borrow
        self.mut_borrows.insert(src_name, Some(dest_name));
        
        Ok(())
    }
    
    fn drop_value(&mut self, value: crate::ir::IrValue) -> Result<(), String> {
        let value_name = self.value_name(&value);
        
        // Check: value should not have active borrows
        if let Some(borrows) = self.imm_borrows.get(&value_name) {
            if !borrows.is_empty() {
                return Err(format!("Cannot drop {}: has active immutable borrows", value_name));
            }
        }
        
        if let Some(Some(_)) = self.mut_borrows.get(&value_name) {
            return Err(format!("Cannot drop {}: has active mutable borrow", value_name));
        }
        
        // Remove from tracking
        self.owners.remove(&value_name);
        self.imm_borrows.remove(&value_name);
        self.mut_borrows.remove(&value_name);
        self.moved.remove(&value_name);
        
        Ok(())
    }
    
    fn value_name(&self, value: &crate::ir::IrValue) -> String {
        match value {
            crate::ir::IrValue::Variable(name) => name.clone(),
            crate::ir::IrValue::Temporary(n) => format!("temp_{}", n),
            crate::ir::IrValue::Constant(_) => "const".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_profile_passes,
        lint_missing_library_imports,
        resolve_imports,
        validate_imports,
        validate_ir_import_contract,
    };
    use crate::ast::{Item, Program};
    use crate::ir::{
        IntType, IrBlock, IrFunction, IrInstruction, IrLiteral, IrModule, IrParameter, IrType,
        IrValue,
    };
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::profile::Profile;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn module_with_instructions(instructions: Vec<IrInstruction>) -> IrModule {
        IrModule {
            version: "0.1".to_string(),
            imports: vec![],
            functions: vec![IrFunction {
                name: "main".to_string(),
                params: vec![IrParameter {
                    name: "arr".to_string(),
                    ty: IrType::Array {
                        inner: Box::new(IrType::Int(IntType::I64)),
                        size: None,
                    },
                }],
                return_type: None,
                body: IrBlock { instructions },
                is_unsafe: false,
            }],
            types: vec![],
            structs: vec![],
            enums: vec![],
            extern_functions: vec![],
        }
    }

    fn parse_program(source: &str) -> Program {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("tokenization must succeed");
        let mut parser = Parser::new(tokens);
        parser.parse().expect("parse must succeed")
    }

    /// Create Label + Branch instructions that form an infinite loop.
    /// Appended to kernel/baremetal entrypoint bodies to satisfy divergence check.
    fn diverging_loop() -> Vec<IrInstruction> {
        vec![
            IrInstruction::Label { name: "_loop".to_string() },
            IrInstruction::Branch { label: "_loop".to_string() },
        ]
    }

    /// Append diverging loop to an instruction vec
    fn with_loop(mut insts: Vec<IrInstruction>) -> Vec<IrInstruction> {
        insts.extend(diverging_loop());
        insts
    }

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be monotonic")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("falcon_import_tests_{}_{}", name, stamp));
        fs::create_dir_all(&dir).expect("temp directory must be creatable");
        dir
    }

    #[test]
    fn userland_bounds_check_uses_known_upper_bound_from_array_init() {
        let mut module = module_with_instructions(vec![
            IrInstruction::ArrayInit {
                elements: vec![
                    IrValue::Constant(IrLiteral::Int(1)),
                    IrValue::Constant(IrLiteral::Int(2)),
                    IrValue::Constant(IrLiteral::Int(3)),
                ],
                dest: IrValue::Temporary(0),
            },
            IrInstruction::Index {
                base: IrValue::Temporary(0),
                index: IrValue::Constant(IrLiteral::Int(1)),
                dest: IrValue::Temporary(1),
            },
        ]);

        apply_profile_passes(&mut module, Profile::Userland).expect("userland pass must succeed");
        let body = &module.functions[0].body.instructions;

        assert!(matches!(
            body.get(1),
            Some(IrInstruction::BoundsCheck {
                bound: IrValue::Constant(IrLiteral::Int(3)),
                ..
            })
        ));
    }

    #[test]
    fn userland_bounds_check_propagates_length_through_move() {
        let mut module = module_with_instructions(vec![
            IrInstruction::ArrayInit {
                elements: vec![
                    IrValue::Constant(IrLiteral::Int(10)),
                    IrValue::Constant(IrLiteral::Int(20)),
                ],
                dest: IrValue::Temporary(0),
            },
            IrInstruction::Move {
                src: IrValue::Temporary(0),
                dest: IrValue::Variable("local_arr".to_string()),
            },
            IrInstruction::Index {
                base: IrValue::Variable("local_arr".to_string()),
                index: IrValue::Constant(IrLiteral::Int(0)),
                dest: IrValue::Temporary(1),
            },
        ]);

        apply_profile_passes(&mut module, Profile::Userland).expect("userland pass must succeed");
        let body = &module.functions[0].body.instructions;

        assert!(matches!(
            body.get(2),
            Some(IrInstruction::BoundsCheck {
                bound: IrValue::Constant(IrLiteral::Int(2)),
                ..
            })
        ));
    }

    #[test]
    fn userland_bounds_check_uses_unknown_sentinel_when_length_unavailable() {
        let mut module = module_with_instructions(vec![IrInstruction::Index {
            base: IrValue::Variable("arr".to_string()),
            index: IrValue::Constant(IrLiteral::Int(0)),
            dest: IrValue::Temporary(0),
        }]);

        apply_profile_passes(&mut module, Profile::Userland).expect("userland pass must succeed");
        let body = &module.functions[0].body.instructions;

        assert!(matches!(
            body.get(0),
            Some(IrInstruction::BoundsCheck {
                bound: IrValue::Constant(IrLiteral::Int(-1)),
                ..
            })
        ));
    }

    #[test]
    fn kernel_profile_does_not_insert_bounds_checks() {
        let mut module = module_with_instructions(with_loop(vec![IrInstruction::Index {
            base: IrValue::Variable("arr".to_string()),
            index: IrValue::Constant(IrLiteral::Int(0)),
            dest: IrValue::Temporary(0),
        }]));
        module.functions[0].name = "kernel_main".to_string();
        module.extern_functions.push("hw_init".to_string());

        apply_profile_passes(&mut module, Profile::Kernel).expect("kernel pass must succeed");
        let body = &module.functions[0].body.instructions;

        assert!(!body.iter().any(|inst| matches!(inst, IrInstruction::BoundsCheck { .. })));
    }

    #[test]
    fn validate_imports_rejects_profile_routed_module_in_kernel() {
        let ast = parse_program(
            r#"
import random;
func main() {}
"#,
        );

        let error = validate_imports(&ast, Profile::Kernel).expect_err("kernel import should fail");
        assert!(error.contains("KERNEL PROFILE VIOLATION"));
    }

    #[test]
    fn validate_imports_allows_raw_import_in_kernel() {
        let ast = parse_program(
            r#"
import random::raw;
func main() {}
"#,
        );

        validate_imports(&ast, Profile::Kernel).expect("raw import should be allowed in kernel");
    }

    #[test]
    fn resolve_imports_expands_grouped_imports_recursively() {
        let dir = temp_dir("grouped_recursive");
        fs::create_dir_all(dir.join("foo")).expect("foo directory must be creatable");

        fs::write(
            dir.join("foo").join("bar.fc"),
            "func bar() { }\n",
        )
        .expect("bar module must be writable");
        fs::write(
            dir.join("foo").join("baz.fc"),
            "func baz() { }\n",
        )
        .expect("baz module must be writable");

        let mut ast = parse_program(
            r#"
import foo::{bar, baz};
func main() {}
"#,
        );

        resolve_imports(&mut ast, &dir, Profile::Userland).expect("grouped import must resolve");

        let function_names: Vec<String> = ast
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Function(function) => Some(function.name.clone()),
                _ => None,
            })
            .collect();

        assert!(function_names.iter().any(|name| name == "bar"));
        assert!(function_names.iter().any(|name| name == "baz"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_imports_detects_cycles() {
        let dir = temp_dir("cycle");

        fs::write(
            dir.join("a.fc"),
            r#"
import b;
func a_func() {}
"#,
        )
        .expect("a.fc must be writable");

        fs::write(
            dir.join("b.fc"),
            r#"
import a;
func b_func() {}
"#,
        )
        .expect("b.fc must be writable");

        let mut ast = parse_program(
            r#"
import a;
func main() {}
"#,
        );

        let error = resolve_imports(&mut ast, &dir, Profile::Userland)
            .expect_err("cyclic imports must fail");
        assert!(error.contains("IMPORT CYCLE DETECTED"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_imports_rejects_unknown_module_without_fallback() {
        let dir = temp_dir("missing");
        let mut ast = parse_program(
            r#"
import this_module_does_not_exist;
func main() {}
"#,
        );

        let error = resolve_imports(&mut ast, &dir, Profile::Userland)
            .expect_err("missing module must fail");
        assert!(error.contains("IMPORT RESOLUTION ERROR"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_imports_profile_legality_matrix() {
        let routed_modules = ["random", "math", "io", "ai"];

        for module in routed_modules {
            let userland_source = format!("import {};\nfunc main() {{}}\n", module);
            let userland_ast = parse_program(&userland_source);
            validate_imports(&userland_ast, Profile::Userland)
                .expect("routed import should be legal in userland");

            let kernel_error = validate_imports(&userland_ast, Profile::Kernel)
                .expect_err("routed import should fail in kernel");
            assert!(kernel_error.contains("KERNEL PROFILE VIOLATION"));

            let baremetal_error = validate_imports(&userland_ast, Profile::Baremetal)
                .expect_err("routed import should fail in baremetal");
            assert!(baremetal_error.contains("BAREMETAL PROFILE VIOLATION"));

            let raw_source = format!("import {}::raw;\nfunc main() {{}}\n", module);
            let raw_ast = parse_program(&raw_source);
            validate_imports(&raw_ast, Profile::Userland)
                .expect("raw import should be legal in userland");
            validate_imports(&raw_ast, Profile::Kernel)
                .expect("raw import should be legal in kernel");
            validate_imports(&raw_ast, Profile::Baremetal)
                .expect("raw import should be legal in baremetal");
        }
    }

    #[test]
    fn kernel_profile_requires_non_returning_kernel_entrypoint() {
        let mut module = module_with_instructions(diverging_loop());
        module.functions[0].name = "kernel_main".to_string();
        module.extern_functions.push("hw_init".to_string());
        apply_profile_passes(&mut module, Profile::Kernel).expect("kernel_main without return must be valid");

        let mut missing_entry = module_with_instructions(vec![]);
        missing_entry.extern_functions.push("hw_init".to_string());
        let error = apply_profile_passes(&mut missing_entry, Profile::Kernel)
            .expect_err("kernel profile should require kernel entrypoint");
        assert!(error.contains("Missing entrypoint"));

        let mut returning_entry = module_with_instructions(vec![IrInstruction::Return { value: None }]);
        returning_entry.functions[0].name = "kernel_main".to_string();
        returning_entry.extern_functions.push("hw_init".to_string());
        let error = apply_profile_passes(&mut returning_entry, Profile::Kernel)
            .expect_err("kernel entrypoint must not return");
        assert!(error.contains("must not return"));
    }

    #[test]
    fn baremetal_profile_requires_non_returning_start_and_rejects_print() {
        let mut module = module_with_instructions(diverging_loop());
        module.functions[0].name = "_start".to_string();
        module.extern_functions.push("hw_init".to_string());
        apply_profile_passes(&mut module, Profile::Baremetal).expect("_start without return must be valid");

        let mut missing_entry = module_with_instructions(vec![]);
        missing_entry.extern_functions.push("hw_init".to_string());
        let error = apply_profile_passes(&mut missing_entry, Profile::Baremetal)
            .expect_err("baremetal profile should require _start");
        assert!(error.contains("Missing entrypoint"));

        let mut returning_entry = module_with_instructions(vec![IrInstruction::Return { value: None }]);
        returning_entry.functions[0].name = "_start".to_string();
        returning_entry.extern_functions.push("hw_init".to_string());
        let error = apply_profile_passes(&mut returning_entry, Profile::Baremetal)
            .expect_err("baremetal _start must not return");
        assert!(error.contains("must not return"));

        let mut hosted_io = module_with_instructions(with_loop(vec![IrInstruction::Call {
            func: "println".to_string(),
            args: vec![],
            dest: None,
        }]));
        hosted_io.functions[0].name = "_start".to_string();
        hosted_io.extern_functions.push("hw_init".to_string());
        let error = apply_profile_passes(&mut hosted_io, Profile::Baremetal)
            .expect_err("baremetal should reject hosted io");
        assert!(error.contains("runtime call 'println'"));
    }

    #[test]
    fn kernel_profile_rejects_runtime_os_exec_call() {
        let mut module = module_with_instructions(with_loop(vec![IrInstruction::Call {
            func: "falcon_os_exec_capture".to_string(),
            args: vec![IrValue::Constant(IrLiteral::String("echo hi".to_string()))],
            dest: Some(IrValue::Temporary(0)),
        }]));
        module.functions[0].name = "kernel_main".to_string();

        let error = apply_profile_passes(&mut module, Profile::Kernel)
            .expect_err("kernel should reject hosted os/runtime helper calls");
        assert!(error.contains("runtime call 'falcon_os_exec_capture'"));
        assert!(error.contains("KERNEL PROFILE VIOLATION"));
    }

    #[test]
    fn baremetal_profile_rejects_runtime_ai_call() {
        let mut module = module_with_instructions(with_loop(vec![IrInstruction::Call {
            func: "falcon_ollama_generate".to_string(),
            args: vec![
                IrValue::Constant(IrLiteral::String("llama3.1:8b".to_string())),
                IrValue::Constant(IrLiteral::String("hello".to_string())),
            ],
            dest: Some(IrValue::Temporary(0)),
        }]));
        module.functions[0].name = "_start".to_string();

        let error = apply_profile_passes(&mut module, Profile::Baremetal)
            .expect_err("baremetal should reject hosted ai/runtime helper calls");
        assert!(error.contains("runtime call 'falcon_ollama_generate'"));
        assert!(error.contains("BAREMETAL PROFILE VIOLATION"));
    }

    #[test]
    fn resolve_imports_returns_ir_metadata() {
        let dir = temp_dir("import_metadata");
        fs::create_dir_all(dir.join("random")).expect("random directory must be creatable");
        fs::write(
            dir.join("random").join("mod.fc"),
            "extern func falcon_randint(a: i64, b: i64) -> i64;\n",
        )
        .expect("random mod must be writable");

        let mut ast = parse_program(
            r#"
import random;
func main() {}
"#,
        );

        let imports =
            resolve_imports(&mut ast, &dir, Profile::Userland).expect("import resolution must succeed");
        assert!(imports.iter().any(|import| {
            import.module == "random"
                && import.resolved_to == "random::mod"
                && import.profile == "userland"
        }));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn strict_import_lint_and_ir_contract_require_explicit_module_import() {
        let mut module = module_with_instructions(vec![IrInstruction::Call {
            func: "falcon_randint".to_string(),
            args: vec![
                IrValue::Constant(IrLiteral::Int(1)),
                IrValue::Constant(IrLiteral::Int(10)),
            ],
            dest: Some(IrValue::Temporary(0)),
        }]);

        let lint_error = lint_missing_library_imports(&module)
            .expect_err("lint must reject missing random import");
        assert!(lint_error.contains("requires `import random`"));

        let contract_error = validate_ir_import_contract(&module)
            .expect_err("ir contract must reject missing random import");
        assert!(contract_error.contains("IR IMPORT CONTRACT VIOLATION"));

        module.imports.push(crate::ir::IrImportResolution {
            module: "random".to_string(),
            resolved_to: "random::mod".to_string(),
            profile: "userland".to_string(),
        });

        lint_missing_library_imports(&module).expect("lint should pass with import");
        validate_ir_import_contract(&module).expect("ir contract should pass with import");
    }
}

/// Validate that all trait implementations provide the required methods.
///
/// For each `impl Trait for Type`, verifies:
/// - The trait exists
/// - All required methods are implemented
/// - No trait methods are missing
pub fn validate_trait_impls(ast: &crate::ast::Program) -> Result<(), String> {
    use std::collections::HashMap;

    // Collect all trait definitions: trait_name -> Vec<method_name>
    let mut traits: HashMap<String, Vec<String>> = HashMap::new();
    for item in &ast.items {
        if let crate::ast::Item::Trait(t) = item {
            let method_names: Vec<String> = t.methods.iter()
                .map(|m| m.name.clone())
                .collect();
            traits.insert(t.name.clone(), method_names);
        }
    }

    // Check each impl block that implements a trait
    for item in &ast.items {
        if let crate::ast::Item::Impl(impl_block) = item {
            if let Some(ref trait_name) = impl_block.trait_name {
                // Look up the trait
                let Some(required_methods) = traits.get(trait_name) else {
                    return Err(format!(
                        "TRAIT ERROR: `impl {} for {}` — trait `{}` is not defined",
                        trait_name, impl_block.type_name, trait_name
                    ));
                };

                // Collect implemented method names
                let implemented: Vec<String> = impl_block.methods.iter()
                    .map(|m| m.name.clone())
                    .collect();

                // Check for missing methods
                let missing: Vec<&String> = required_methods.iter()
                    .filter(|name| !implemented.contains(name))
                    .collect();

                if !missing.is_empty() {
                    let missing_list = missing.iter()
                        .map(|n| format!("  - {}", n))
                        .collect::<Vec<_>>()
                        .join("\n");
                    return Err(format!(
                        "TRAIT ERROR: `impl {} for {}` is missing required methods:\n{}",
                        trait_name, impl_block.type_name, missing_list
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Monomorphize generic functions at the AST level.
///
/// For each generic function `func foo<T>(x: T) -> T { ... }` and each
/// call site `foo(42)`, infer concrete types from arguments, create a
/// specialized copy `foo__i64`, and rewrite the call site.
///
/// This runs BEFORE IR lowering so the IR never sees TypeParam.
pub fn monomorphize_generics(ast: &mut crate::ast::Program) {
    use crate::ast::*;
    use std::collections::HashMap;

    // 1. Collect generic function templates: name -> Function
    let generic_templates: HashMap<String, Function> = ast
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Function(f) = item {
                if !f.type_params.is_empty() {
                    return Some((f.name.clone(), f.clone()));
                }
            }
            None
        })
        .collect();

    if generic_templates.is_empty() {
        return;
    }

    // 2. Collect call sites and infer concrete types
    //    For simplicity, we infer from arguments: Int literal -> i64, Float -> f64, Bool -> bool
    //    TODO: extend with full type inference from variable types
    let mut specializations: HashMap<String, Vec<(String, Vec<(String, Type)>)>> = HashMap::new();

    fn infer_type_from_expr(expr: &Expression) -> Option<Type> {
        match expr {
            Expression::Literal(Literal::Int(_)) => Some(Type::Int(IntType::I64)),
            Expression::Literal(Literal::Float(_)) => Some(Type::Float(FloatType::F64)),
            Expression::Literal(Literal::Bool(_)) => Some(Type::Bool),
            Expression::Literal(Literal::String(_)) => Some(Type::String),
            Expression::Literal(Literal::Char(_)) => Some(Type::Int(IntType::I8)),
            _ => None, // Cannot infer — caller must handle
        }
    }

    fn type_suffix(ty: &Type) -> String {
        match ty {
            Type::Int(IntType::I8) => "i8".to_string(),
            Type::Int(IntType::I16) => "i16".to_string(),
            Type::Int(IntType::I32) => "i32".to_string(),
            Type::Int(IntType::I64) => "i64".to_string(),
            Type::Int(IntType::U8) => "u8".to_string(),
            Type::Int(IntType::U16) => "u16".to_string(),
            Type::Int(IntType::U32) => "u32".to_string(),
            Type::Int(IntType::U64) => "u64".to_string(),
            Type::Float(FloatType::F32) => "f32".to_string(),
            Type::Float(FloatType::F64) => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
            Type::Named(n) => n.clone(),
            _ => "i64".to_string(),
        }
    }

    fn collect_call_specializations(
        expr: &Expression,
        generic_templates: &HashMap<String, Function>,
        specializations: &mut HashMap<String, Vec<(String, Vec<(String, Type)>)>>,
    ) {
        match expr {
            Expression::Call { callee, args, type_args } => {
                if let Expression::Variable(name) = callee.as_ref() {
                    if let Some(template) = generic_templates.get(name) {
                        // Infer type bindings from arguments
                        let mut bindings: Vec<(String, Type)> = Vec::new();

                        if !type_args.is_empty() {
                            // Explicit type args: identity::<i64>(42)
                            for (i, tp) in template.type_params.iter().enumerate() {
                                if i < type_args.len() {
                                    bindings.push((tp.clone(), type_args[i].clone()));
                                }
                            }
                        } else {
                            // Infer from argument types
                            for (i, param) in template.params.iter().enumerate() {
                                // Parser may store type params as TypeParam("T") or Named("T")
                                let tp_name = match &param.ty {
                                    Type::TypeParam(n) => Some(n.clone()),
                                    Type::Named(n) if template.type_params.contains(n) => Some(n.clone()),
                                    _ => None,
                                };
                                if let Some(tp_name) = tp_name {
                                    if i < args.len() {
                                        if let Some(inferred) = infer_type_from_expr(&args[i]) {
                                            if !bindings.iter().any(|(n, _)| n == &tp_name) {
                                                bindings.push((tp_name, inferred));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Reject unresolved type params — no guessing
                        let mut all_resolved = true;
                        for tp in &template.type_params {
                            if !bindings.iter().any(|(n, _)| n == tp) {
                                eprintln!("error: cannot infer generic parameter '{}' in '{}'. \
                                          Provide explicit type arguments.", tp, template.name);
                                all_resolved = false;
                            }
                        }
                        if !all_resolved {
                            return; // Can't generate specialization — types unknown
                        }

                        // Generate mangled name
                        let suffix: Vec<String> = template
                            .type_params
                            .iter()
                            .map(|tp| {
                                bindings
                                    .iter()
                                    .find(|(n, _)| n == tp)
                                    .map(|(_, ty)| type_suffix(ty))
                                    .unwrap_or_else(|| "i64".to_string())
                            })
                            .collect();
                        let mangled = format!("{}_{}", name, suffix.join("_"));

                        specializations
                            .entry(name.clone())
                            .or_insert_with(Vec::new)
                            .push((mangled, bindings));
                    }
                }
                // Recurse into callee and args
                collect_call_specializations(callee, generic_templates, specializations);
                for arg in args {
                    collect_call_specializations(arg, generic_templates, specializations);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                collect_call_specializations(left, generic_templates, specializations);
                collect_call_specializations(right, generic_templates, specializations);
            }
            Expression::UnaryOp { operand, .. } => {
                collect_call_specializations(operand, generic_templates, specializations);
            }
            Expression::Block(block) => {
                for stmt in &block.statements {
                    collect_stmt_specializations(stmt, generic_templates, specializations);
                }
            }
            Expression::If(if_expr) => {
                collect_call_specializations(&if_expr.condition, generic_templates, specializations);
                collect_call_specializations(&if_expr.then_expr, generic_templates, specializations);
                collect_call_specializations(&if_expr.else_expr, generic_templates, specializations);
            }
            _ => {}
        }
    }

    fn collect_stmt_specializations(
        stmt: &Statement,
        generic_templates: &HashMap<String, Function>,
        specializations: &mut HashMap<String, Vec<(String, Vec<(String, Type)>)>>,
    ) {
        match stmt {
            Statement::Let(let_stmt) => {
                collect_call_specializations(&let_stmt.value, generic_templates, specializations);
            }
            Statement::Assign(assign) => {
                collect_call_specializations(&assign.value, generic_templates, specializations);
            }
            Statement::Expr(expr) => {
                collect_call_specializations(expr, generic_templates, specializations);
            }
            Statement::Return(Some(expr)) => {
                collect_call_specializations(expr, generic_templates, specializations);
            }
            Statement::If(if_stmt) => {
                collect_call_specializations(&if_stmt.condition, generic_templates, specializations);
                for stmt in &if_stmt.then_block.statements {
                    collect_stmt_specializations(stmt, generic_templates, specializations);
                }
                if let Some(else_block) = &if_stmt.else_block {
                    for stmt in &else_block.statements {
                        collect_stmt_specializations(stmt, generic_templates, specializations);
                    }
                }
            }
            Statement::While(while_stmt) => {
                collect_call_specializations(&while_stmt.condition, generic_templates, specializations);
                for stmt in &while_stmt.body.statements {
                    collect_stmt_specializations(stmt, generic_templates, specializations);
                }
            }
            Statement::For(for_stmt) => {
                collect_call_specializations(&for_stmt.iterable, generic_templates, specializations);
                for stmt in &for_stmt.body.statements {
                    collect_stmt_specializations(stmt, generic_templates, specializations);
                }
            }
            Statement::Match(match_stmt) => {
                collect_call_specializations(&match_stmt.expr, generic_templates, specializations);
                for arm in &match_stmt.arms {
                    collect_call_specializations(&arm.body, generic_templates, specializations);
                }
            }
            _ => {}
        }
    }

    // Scan all functions for call sites
    for item in &ast.items {
        if let Item::Function(f) = item {
            for stmt in &f.body.statements {
                collect_stmt_specializations(stmt, &generic_templates, &mut specializations);
            }
        }
        // Also check impl methods
        if let Item::Impl(impl_block) = item {
            for method in &impl_block.methods {
                for stmt in &method.body.statements {
                    collect_stmt_specializations(stmt, &generic_templates, &mut specializations);
                }
            }
        }
    }

    // 3. Generate specialized functions
    fn substitute_type(ty: &Type, bindings: &[(String, Type)]) -> Type {
        match ty {
            Type::TypeParam(name) | Type::Named(name) if bindings.iter().any(|(n, _)| n == name) => {
                bindings
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, t)| t.clone())
                    .unwrap() // Safe: guard ensures binding exists
            }
            Type::Pointer { mutable, inner } => Type::Pointer {
                mutable: *mutable,
                inner: Box::new(substitute_type(inner, bindings)),
            },
            Type::Reference { mutable, lifetime, inner } => Type::Reference {
                mutable: *mutable,
                lifetime: lifetime.clone(),
                inner: Box::new(substitute_type(inner, bindings)),
            },
            Type::Array { inner, size } => Type::Array {
                inner: Box::new(substitute_type(inner, bindings)),
                size: *size,
            },
            Type::Function { params, return_type } => Type::Function {
                params: params.iter().map(|p| substitute_type(p, bindings)).collect(),
                return_type: Box::new(substitute_type(return_type, bindings)),
            },
            other => other.clone(),
        }
    }

    let mut new_functions: Vec<Item> = Vec::new();
    let mut seen_mangled: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (generic_name, instances) in &specializations {
        if let Some(template) = generic_templates.get(generic_name) {
            for (mangled_name, bindings) in instances {
                if seen_mangled.contains(mangled_name) {
                    continue;
                }
                seen_mangled.insert(mangled_name.clone());

                // Clone the template and substitute types
                let mut specialized = template.clone();
                specialized.name = mangled_name.clone();
                specialized.type_params.clear(); // No longer generic

                // Substitute parameter types
                for param in &mut specialized.params {
                    param.ty = substitute_type(&param.ty, bindings);
                }

                // Substitute return type
                if let Some(ref rt) = specialized.return_type {
                    specialized.return_type = Some(substitute_type(rt, bindings));
                }

                new_functions.push(Item::Function(specialized));
            }
        }
    }

    // 4. Rewrite call sites
    fn rewrite_calls_in_expr(
        expr: &mut Expression,
        generic_templates: &HashMap<String, Function>,
        specializations: &HashMap<String, Vec<(String, Vec<(String, Type)>)>>,
    ) {
        match expr {
            Expression::Call { callee, args, type_args } => {
                // Recurse first
                rewrite_calls_in_expr(callee, generic_templates, specializations);
                for arg in args.iter_mut() {
                    rewrite_calls_in_expr(arg, generic_templates, specializations);
                }

                // Rewrite if callee is a generic function
                if let Expression::Variable(name) = callee.as_ref() {
                    if let Some(template) = generic_templates.get(name) {
                        if let Some(instances) = specializations.get(name) {
                            // Re-infer bindings for THIS call to find the correct specialization
                            let mut call_bindings: Vec<(String, Type)> = Vec::new();
                            if !type_args.is_empty() {
                                for (i, tp) in template.type_params.iter().enumerate() {
                                    if i < type_args.len() {
                                        call_bindings.push((tp.clone(), type_args[i].clone()));
                                    }
                                }
                            } else {
                                for (i, param) in template.params.iter().enumerate() {
                                    let tp_name = match &param.ty {
                                        Type::TypeParam(n) => Some(n.clone()),
                                        Type::Named(n) if template.type_params.contains(n) => Some(n.clone()),
                                        _ => None,
                                    };
                                    if let Some(tp_name) = tp_name {
                                        if i < args.len() {
                                            if let Some(inferred) = infer_type_from_expr(&args[i]) {
                                                if !call_bindings.iter().any(|(n, _)| n == &tp_name) {
                                                    call_bindings.push((tp_name, inferred));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            // If any type param is unresolved, skip rewrite — don't guess
                            let all_resolved = template.type_params.iter()
                                .all(|tp| call_bindings.iter().any(|(n, _)| n == tp));
                            if !all_resolved {
                                return; // Can't determine all types — leave call as-is
                            }
                            // Build expected mangled name
                            let suffix: Vec<String> = template.type_params.iter().map(|tp| {
                                call_bindings.iter()
                                    .find(|(n, _)| n == tp)
                                    .map(|(_, ty)| type_suffix(ty))
                                    .unwrap_or_else(|| "i64".to_string())
                            }).collect();
                            let expected_mangled = format!("{}_{}", name, suffix.join("_"));
                            
                            // Find matching instance, or fall back to first
                            let mangled = instances.iter()
                                .find(|(m, _)| m == &expected_mangled)
                                .map(|(m, _)| m.clone())
                                .unwrap_or_else(|| instances.first().map(|(m, _)| m.clone()).unwrap_or_default());
                            
                            if !mangled.is_empty() {
                                *callee = Box::new(Expression::Variable(mangled));
                            }
                        }
                    }
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                rewrite_calls_in_expr(left, generic_templates, specializations);
                rewrite_calls_in_expr(right, generic_templates, specializations);
            }
            Expression::UnaryOp { operand, .. } => {
                rewrite_calls_in_expr(operand, generic_templates, specializations);
            }
            Expression::Block(block) => {
                for stmt in &mut block.statements {
                    rewrite_calls_in_stmt(stmt, generic_templates, specializations);
                }
            }
            Expression::If(if_expr) => {
                rewrite_calls_in_expr(&mut if_expr.condition, generic_templates, specializations);
                rewrite_calls_in_expr(&mut if_expr.then_expr, generic_templates, specializations);
                rewrite_calls_in_expr(&mut if_expr.else_expr, generic_templates, specializations);
            }
            _ => {}
        }
    }

    fn rewrite_calls_in_stmt(
        stmt: &mut Statement,
        generic_templates: &HashMap<String, Function>,
        specializations: &HashMap<String, Vec<(String, Vec<(String, Type)>)>>,
    ) {
        match stmt {
            Statement::Let(let_stmt) => {
                rewrite_calls_in_expr(&mut let_stmt.value, generic_templates, specializations);
            }
            Statement::Assign(assign) => {
                rewrite_calls_in_expr(&mut assign.value, generic_templates, specializations);
            }
            Statement::Expr(expr) => {
                rewrite_calls_in_expr(expr, generic_templates, specializations);
            }
            Statement::Return(Some(expr)) => {
                rewrite_calls_in_expr(expr, generic_templates, specializations);
            }
            Statement::If(if_stmt) => {
                rewrite_calls_in_expr(&mut if_stmt.condition, generic_templates, specializations);
                for stmt in &mut if_stmt.then_block.statements {
                    rewrite_calls_in_stmt(stmt, generic_templates, specializations);
                }
                if let Some(else_block) = &mut if_stmt.else_block {
                    for stmt in &mut else_block.statements {
                        rewrite_calls_in_stmt(stmt, generic_templates, specializations);
                    }
                }
            }
            Statement::While(while_stmt) => {
                rewrite_calls_in_expr(&mut while_stmt.condition, generic_templates, specializations);
                for stmt in &mut while_stmt.body.statements {
                    rewrite_calls_in_stmt(stmt, generic_templates, specializations);
                }
            }
            Statement::For(for_stmt) => {
                rewrite_calls_in_expr(&mut for_stmt.iterable, generic_templates, specializations);
                for stmt in &mut for_stmt.body.statements {
                    rewrite_calls_in_stmt(stmt, generic_templates, specializations);
                }
            }
            Statement::Match(match_stmt) => {
                rewrite_calls_in_expr(&mut match_stmt.expr, generic_templates, specializations);
                for arm in &mut match_stmt.arms {
                    rewrite_calls_in_expr(&mut arm.body, generic_templates, specializations);
                }
            }
            _ => {}
        }
    }

    // Apply rewrites to all functions and impl methods
    for item in &mut ast.items {
        match item {
            Item::Function(f) => {
                for stmt in &mut f.body.statements {
                    rewrite_calls_in_stmt(stmt, &generic_templates, &specializations);
                }
            }
            Item::Impl(impl_block) => {
                for method in &mut impl_block.methods {
                    for stmt in &mut method.body.statements {
                        rewrite_calls_in_stmt(stmt, &generic_templates, &specializations);
                    }
                }
            }
            _ => {}
        }
    }

    // 5. Remove original generic templates and add specialized versions
    ast.items.retain(|item| {
        if let Item::Function(f) = item {
            if generic_templates.contains_key(&f.name) {
                return false; // Remove generic template
            }
        }
        true
    });

    ast.items.extend(new_functions);
}
