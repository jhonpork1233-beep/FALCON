use clap::{Parser as ClapParser, Subcommand};
use falcon_compiler::Profile;
use std::path::{Path, PathBuf};
use std::str::FromStr;
#[cfg(feature = "llvm")]
use std::process::Command;

#[derive(ClapParser)]
#[command(name = "falcon")]
#[command(about = "Falcon Programming Language Compiler")]
#[command(version = "0.2.0")]
struct Cli {
    /// Input source file (shortcut for: falcon run <file>)
    /// Example: falcon example.fc
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
    
    /// Compilation profile for direct run (userland, kernel, baremetal)
    #[arg(long, default_value = "userland")]
    profile: String,
    
    /// Build with multiple profiles: all, or comma-separated (userland,kernel,baremetal)
    /// Example: falcon demo.fc --profiles=all
    #[arg(long)]
    profiles: Option<String>,

    /// Keep generated .__gen__.fc files when transpiling .fpy sources
    #[arg(long)]
    keep_generated: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a Falcon program
    Build {
        /// Input source file
        #[arg(required = true)]
        file: PathBuf,
        
        /// Compilation profile (userland, kernel, baremetal)
        #[arg(long, default_value = "userland")]
        profile: String,
        
        /// Output file (default: same as input with .exe/.out extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Optimization level (0-3)
        #[arg(long, default_value = "0")]
        opt: u8,
        
        /// Emit IR instead of binary
        #[arg(long)]
        emit_ir: bool,
        
        /// Emit C code only (don't compile to binary)
        #[arg(long)]
        emit_c: bool,
        
        /// Emit LLVM IR instead of binary
        #[arg(long)]
        emit_llvm: bool,
        
        /// Enable memory sanitizers (userland only)
        #[arg(long)]
        sanitize: Option<String>,
        
        /// Run the program after building
        #[arg(long)]
        run: bool,

        /// Keep generated .__gen__.fc files when transpiling .fpy sources
        #[arg(long)]
        keep_generated: bool,

        /// Enforce explicit library imports for runtime/binding symbols
        #[arg(long)]
        strict_imports: bool,

        /// Print resolved imports (`module -> resolved_to @ profile`)
        #[arg(long)]
        dump_imports: bool,

        /// Cross-compilation target triple (e.g., aarch64-unknown-none, riscv64gc-unknown-none-elf)
        /// Overrides the default target for the selected profile.
        #[arg(long)]
        target: Option<String>,
    },
    
    /// Check source without building (parse + profile/import/IR validation)
    Check {
        /// Input source file
        #[arg(required = true)]
        file: PathBuf,

        /// Compilation profile (userland, kernel, baremetal)
        #[arg(long, default_value = "userland")]
        profile: String,

        /// Keep generated .__gen__.fc files when transpiling .fpy sources
        #[arg(long)]
        keep_generated: bool,

        /// Enforce explicit library imports for runtime/binding symbols
        #[arg(long)]
        strict_imports: bool,

        /// Print resolved imports (`module -> resolved_to @ profile`)
        #[arg(long)]
        dump_imports: bool,
    },
    
    /// Format Falcon code
    Fmt {
        /// Input source file(s)
        files: Vec<PathBuf>,
        
        /// Check formatting without modifying files
        #[arg(long)]
        check: bool,
    },
    
    /// Run a Falcon program in fast/dev mode (userland profile only)
    /// Uses optimized execution path - ideal for scripting and experimentation
    /// Note: kernel and baremetal profiles require 'falcon build --profile=X'
    Run {
        /// Input source file
        #[arg(required = true)]
        file: PathBuf,

        /// Execution profile (only `userland` is allowed for `run`)
        #[arg(long, default_value = "userland")]
        profile: String,
        
        /// Use LLVM JIT execution (fastest, default)
        #[arg(long)]
        jit: bool,
        
        /// Use native compilation (compile+link+run)
        #[arg(long)]
        native: bool,

        /// Keep generated .__gen__.fc files when transpiling .fpy sources
        #[arg(long)]
        keep_generated: bool,
    },
}

/// Get the path to the Falcon runtime directory
fn get_runtime_dir() -> PathBuf {
    // Try to find runtime relative to executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let runtime_dir = exe_dir.join("runtime");
            if runtime_dir.exists() {
                return runtime_dir;
            }
            // Check parent directory (for development builds: target/release)
            if let Some(parent) = exe_dir.parent() {
                let runtime_dir = parent.join("runtime");
                if runtime_dir.exists() {
                    return runtime_dir;
                }
                // Check grandparent directory (for target/release/deps)
                if let Some(grandparent) = parent.parent() {
                    let runtime_dir = grandparent.join("runtime");
                    if runtime_dir.exists() {
                        return runtime_dir;
                    }
                }
            }
        }
    }
    
    // Try FALCON_RUNTIME_DIR environment variable
    if let Ok(runtime_dir) = std::env::var("FALCON_RUNTIME_DIR") {
        let path = PathBuf::from(&runtime_dir);
        if path.exists() {
            return path;
        }
    }
    
    // Fallback: look in current directory
    PathBuf::from("runtime")
}

/// Copy runtime files to output directory
fn runtime_source_filename(profile: Profile) -> &'static str {
    match profile {
        Profile::Userland => "falcon_runtime.c",
        Profile::Kernel => "falcon_runtime_kernel.c",
        Profile::Baremetal => "falcon_runtime_baremetal.c",
    }
}

/// Resolve runtime source and include directory for the active profile.
/// Returns (runtime_c_path, runtime_include_dir).
fn setup_runtime(output_dir: &PathBuf, profile: Profile) -> anyhow::Result<(PathBuf, PathBuf)> {
    let runtime_dir = get_runtime_dir();
    let runtime_h = runtime_dir.join("falcon_runtime.h");
    let runtime_c_name = runtime_source_filename(profile);
    let runtime_c = runtime_dir.join(runtime_c_name);
    
    if runtime_h.exists() && runtime_c.exists() {
        return Ok((runtime_c, runtime_dir));
    }

    // Fallback: write embedded runtime assets to output directory.
    create_embedded_runtime(output_dir, profile)?;
    let embedded_c = output_dir.join(runtime_c_name);
    let embedded_h = output_dir.join("falcon_runtime.h");

    if !embedded_c.exists() || !embedded_h.exists() {
        return Err(anyhow::anyhow!(
            "Failed to materialize embedded runtime files for profile {:?}",
            profile
        ));
    }

    Ok((embedded_c, output_dir.clone()))
}

fn resolve_linker() -> String {
    if let Ok(linker) = std::env::var("FALCON_LINKER") {
        if !linker.trim().is_empty() {
            return linker;
        }
    }

    #[cfg(windows)]
    {
        let candidates = [
            r"C:\Program Files\LLVM\bin\clang.exe",
            r"C:\Program Files (x86)\LLVM\bin\clang.exe",
            r"C:\llvm\bin\clang.exe",
        ];
        for candidate in candidates {
            if std::path::Path::new(candidate).exists() {
                return candidate.to_string();
            }
        }
    }

    "clang".to_string()
}

fn resolve_freestanding_linker() -> String {
    if let Ok(linker) = std::env::var("FALCON_FREESTANDING_LINKER") {
        if !linker.trim().is_empty() {
            return linker;
        }
    }

    #[cfg(windows)]
    {
        let candidates = [
            r"C:\Program Files\LLVM\bin\ld.lld.exe",
            r"C:\Program Files (x86)\LLVM\bin\ld.lld.exe",
            r"C:\llvm\bin\ld.lld.exe",
        ];
        for candidate in candidates {
            if Path::new(candidate).exists() {
                return candidate.to_string();
            }
        }
    }

    "ld.lld".to_string()
}

fn freestanding_target_triple(profile: Profile) -> Option<&'static str> {
    match profile {
        Profile::Userland => None,
        Profile::Kernel | Profile::Baremetal => Some("x86_64-unknown-none-elf"),
    }
}

/// Resolve effective target triple: CLI --target overrides profile default.
fn resolve_target_triple(profile: Profile, cli_target: &Option<String>) -> Option<String> {
    if let Some(target) = cli_target {
        Some(target.clone())
    } else {
        freestanding_target_triple(profile).map(|s| s.to_string())
    }
}

fn linker_script_filename(profile: Profile) -> Option<&'static str> {
    match profile {
        Profile::Kernel => Some("falcon_kernel.ld"),
        Profile::Baremetal => Some("falcon_baremetal.ld"),
        Profile::Userland => None,
    }
}

fn freestanding_entry_symbol(profile: Profile) -> Option<&'static str> {
    match profile {
        Profile::Kernel => Some("_start"),
        Profile::Baremetal => Some("_start"),
        Profile::Userland => None,
    }
}

fn setup_linker_script(output_dir: &PathBuf, profile: Profile) -> anyhow::Result<PathBuf> {
    let script_name = linker_script_filename(profile)
        .ok_or_else(|| anyhow::anyhow!("No linker script required for {:?}", profile))?;

    let runtime_dir = get_runtime_dir();
    let script_path = runtime_dir.join(script_name);
    if script_path.exists() {
        return Ok(script_path);
    }

    // Fallback: write embedded linker script to output directory.
    let script_contents = match profile {
        Profile::Kernel => include_str!("../runtime/falcon_kernel.ld"),
        Profile::Baremetal => include_str!("../runtime/falcon_baremetal.ld"),
        Profile::Userland => {
            return Err(anyhow::anyhow!(
                "No embedded linker script for userland profile"
            ))
        }
    };

    let embedded_script = output_dir.join(script_name);
    std::fs::write(&embedded_script, script_contents).map_err(|e| {
        anyhow::anyhow!(
            "Failed to write embedded linker script {}: {}",
            embedded_script.display(),
            e
        )
    })?;

    Ok(embedded_script)
}

fn resolve_objcopy() -> Option<String> {
    if let Ok(objcopy) = std::env::var("FALCON_OBJCOPY") {
        if !objcopy.trim().is_empty() {
            return Some(objcopy);
        }
    }

    #[cfg(windows)]
    {
        let candidates = [
            r"C:\Program Files\LLVM\bin\llvm-objcopy.exe",
            r"C:\Program Files (x86)\LLVM\bin\llvm-objcopy.exe",
            r"C:\llvm\bin\llvm-objcopy.exe",
        ];
        for candidate in candidates {
            if Path::new(candidate).exists() {
                return Some(candidate.to_string());
            }
        }
    }

    Some("llvm-objcopy".to_string())
}

/// Create embedded runtime files (fallback if runtime directory not found)
fn create_embedded_runtime(output_dir: &PathBuf, profile: Profile) -> anyhow::Result<()> {
    let runtime_h = include_str!("../runtime/falcon_runtime.h");
    let runtime_c = match profile {
        Profile::Userland => include_str!("../runtime/falcon_runtime.c"),
        Profile::Kernel => include_str!("../runtime/falcon_runtime_kernel.c"),
        Profile::Baremetal => include_str!("../runtime/falcon_runtime_baremetal.c"),
    };
    let runtime_c_name = runtime_source_filename(profile);
    
    std::fs::write(output_dir.join("falcon_runtime.h"), runtime_h)?;
    std::fs::write(output_dir.join(runtime_c_name), runtime_c)?;
    
    Ok(())
}

fn is_fpy_source(file: &PathBuf) -> bool {
    file.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("fpy"))
        .unwrap_or(false)
}

fn generated_fc_path(file: &PathBuf) -> anyhow::Result<PathBuf> {
    let stem = file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid source file name: {}", file.display()))?;
    Ok(file.with_file_name(format!("{}.__gen__.fc", stem)))
}

struct PreparedInput {
    compile_file: PathBuf,
    cleanup_generated: Option<PathBuf>,
}

struct GeneratedFileCleanup {
    generated_path: Option<PathBuf>,
}

impl GeneratedFileCleanup {
    fn new(generated_path: Option<PathBuf>) -> Self {
        Self { generated_path }
    }
}

impl Drop for GeneratedFileCleanup {
    fn drop(&mut self) {
        if let Some(path) = self.generated_path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn prepare_compilation_input(
    file: &PathBuf,
    profile: Profile,
    keep_generated: bool,
) -> anyhow::Result<PreparedInput> {
    if !is_fpy_source(file) {
        return Ok(PreparedInput {
            compile_file: file.clone(),
            cleanup_generated: None,
        });
    }

    if profile != Profile::Userland {
        return Err(anyhow::anyhow!(
            "Python-style .fpy is userland-only. Requested profile: {:?}",
            profile
        ));
    }

    let source = std::fs::read_to_string(file)
        .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file.display(), e))?;
    let generated_file = generated_fc_path(file)?;
    let transpiled = falcon_fpy_transpiler::transpile_source(&source, &file.display().to_string())
        .map_err(|e| anyhow::anyhow!("Transpiler error: {}", e))?;

    std::fs::write(&generated_file, transpiled).map_err(|e| {
        anyhow::anyhow!(
            "Failed to write generated Falcon file {}: {}",
            generated_file.display(),
            e
        )
    })?;

    eprintln!(
        "  ⚙ Transpiled {} -> {}",
        file.display(),
        generated_file.display()
    );

    Ok(PreparedInput {
        compile_file: generated_file.clone(),
        cleanup_generated: if keep_generated {
            None
        } else {
            Some(generated_file)
        },
    })
}

/// Run a Falcon program using LLVM JIT (no clang, no object files)
/// This is the fast/dev mode - userland only
#[cfg(feature = "llvm")]
fn run_falcon_jit(file: PathBuf, keep_generated: bool) -> anyhow::Result<()> {
    use falcon_compiler::*;
    
    let start = std::time::Instant::now();
    let profile = Profile::Userland;
    let prepared = prepare_compilation_input(&file, profile, keep_generated)?;
    let _cleanup = GeneratedFileCleanup::new(prepared.cleanup_generated.clone());
    let compile_file = prepared.compile_file;
    
    // Read source
    let source = std::fs::read_to_string(&compile_file)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
    
    // Lex
    let mut lexer = Lexer::new(&source);
    let (tokens, spans) = lexer.tokenize_with_spans()
        .map_err(|e| anyhow::anyhow!("Lexer error: {}", e))?;
    
    // Parse
    let mut parser = Parser::new_with_spans(tokens, spans);
    let mut ast = parser.parse()
        .map_err(|e| anyhow::anyhow!("Parser error: {}", e))?;
    
    passes::filter_ast_by_profile(&mut ast, profile);
    
    // Resolve imports
    let source_dir = compile_file.parent().unwrap_or(std::path::Path::new("."));
    let resolved_imports = passes::resolve_imports(&mut ast, source_dir, profile)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    passes::validate_imports(&ast, profile)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    passes::validate_trait_impls(&ast)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Monomorphize generics before IR lowering
    passes::monomorphize_generics(&mut ast);
    
    // Generate IR
    let mut ir_module = ir::ast_to_ir(&ast)
        .map_err(|e| anyhow::anyhow!("IR error: {}", e))?;
    ir_module.imports = resolved_imports;
    // Strict imports enforced by default — all falcon_* symbols require explicit import
    passes::lint_missing_library_imports(&ir_module)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    passes::validate_ir_import_contract(&ir_module)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Compile to LLVM
    let mut codegen = LlvmCodegen::new("falcon_jit");
    if let Err(e) = codegen.compile(&ir_module) {
        unsafe { codegen.shutdown(); }
        return Err(anyhow::anyhow!("LLVM error: {}", e));
    }
    
    let compile_time = start.elapsed();
    eprintln!("  ⚡ Compiled in {:.0?}", compile_time);
    
    // Execute via JIT
    eprintln!("🚀 Running (JIT)...");
    let exit_code = match codegen.execute_jit() {
        Ok(code) => code,
        Err(e) => {
            unsafe { codegen.shutdown(); }
            return Err(anyhow::anyhow!("JIT error: {}", e));
        }
    };
    
    eprintln!("📊 Exit code: {}", exit_code);
    unsafe { codegen.shutdown(); }
    
    Ok(())
}


fn build_falcon_program(
    file: PathBuf,
    profile: String,
    output: Option<PathBuf>,
    opt: u8,
    emit_ir: bool,
    emit_c: bool,
    emit_llvm: bool,
    run: bool,
    keep_generated: bool,
    strict_imports: bool,
    dump_imports: bool,
    target: Option<String>,
) -> anyhow::Result<()> {
    use falcon_compiler::*;
    
    eprintln!("🦅 Building {} with profile: {}", file.display(), profile);

    // Parse profile
    let profile_enum = Profile::from_str(&profile)
        .map_err(|e| anyhow::anyhow!("Invalid profile: {}", e))?;

    // .fpy pipeline: transpile to generated .fc before entering compiler pipeline.
    let prepared = prepare_compilation_input(&file, profile_enum, keep_generated)?;
    let _cleanup = GeneratedFileCleanup::new(prepared.cleanup_generated.clone());
    let compile_file = prepared.compile_file;
    
    // Read source file
    let source = std::fs::read_to_string(&compile_file)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
    
    // Lex
    let mut lexer = Lexer::new(&source);
    let (tokens, spans) = lexer.tokenize_with_spans()
        .map_err(|e| anyhow::anyhow!("Lexer error: {}", e))?;
    
    // Parse
    let mut parser = Parser::new_with_spans(tokens, spans);
    let mut ast = parser.parse()
        .map_err(|e| anyhow::anyhow!("Parser error: {}", e))?;
    
    // Filter AST by profile (compile-time code erasure)
    // Functions marked with #[other_profile] are removed
    passes::filter_ast_by_profile(&mut ast, profile_enum);
    
    // Resolve imports - load and merge imported files
    let source_dir = compile_file.parent().unwrap_or(std::path::Path::new("."));
    let resolved_imports = passes::resolve_imports(&mut ast, source_dir, profile_enum)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Validate imports against profile capabilities
    // Each module has required capabilities, profiles restrict which are allowed
    passes::validate_imports(&ast, profile_enum)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    passes::validate_trait_impls(&ast)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Monomorphize generics before IR lowering
    passes::monomorphize_generics(&mut ast);
    
    // Convert to IR
    let mut ir = ir::ast_to_ir(&ast)
        .map_err(|e| {
            let _ = std::fs::write("ir_error.txt", format!("{}", e));
            anyhow::anyhow!("IR generation error: {}", e)
        })?;
    ir.imports = resolved_imports;

    if dump_imports {
        if ir.imports.is_empty() {
            eprintln!("  • Resolved imports: (none)");
        } else {
            eprintln!("  • Resolved imports:");
            for import in &ir.imports {
                eprintln!(
                    "    - {} -> {} @ {}",
                    import.module, import.resolved_to, import.profile
                );
            }
        }
    }

    // Strict imports enforced by default — all falcon_* symbols require explicit import
    passes::lint_missing_library_imports(&ir)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    passes::validate_ir_import_contract(&ir)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Apply profile-specific passes
    passes::apply_profile_passes(&mut ir, profile_enum)
        .map_err(|e| anyhow::anyhow!("Profile pass error: {}\n\nThis is a compile-time enforcement. Fix your code.", e))?;
    
    // Verify ownership
    passes::verify_ownership(&ir)
        .map_err(|e| anyhow::anyhow!("Ownership verification error: {}", e))?;
    
    passes::validate_function_calls(&ir)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    eprintln!("  ✓ Parsed and validated");
    
    if emit_ir {
        // Emit IR as JSON
        let ir_json = serde_json::to_string_pretty(&ir)
            .map_err(|e| anyhow::anyhow!("Failed to serialize IR: {}", e))?;
        println!("{}", ir_json);
        return Ok(());
    }
    
    // Emit LLVM IR only (for debugging)
    #[cfg(feature = "llvm")]
    if emit_llvm {
        let module_name = file.file_stem().unwrap_or(std::ffi::OsStr::new("module")).to_string_lossy();
        let mut codegen = falcon_compiler::LlvmCodegen::new(&module_name);
        
        if let Err(e) = codegen.compile(&ir) {
            unsafe { codegen.shutdown(); }
            return Err(anyhow::anyhow!("LLVM compilation error: {}", e));
        }
        
        if opt > 0 {
            if let Err(e) = codegen.optimize(opt) {
                unsafe { codegen.shutdown(); }
                return Err(anyhow::anyhow!("LLVM optimization error: {}", e));
            }
        }

        let llvm_ir = codegen.dump_module();
        unsafe { codegen.shutdown(); }
        println!("{}", llvm_ir);
        return Ok(());
    }
    
    // Emit C code only (legacy/debug)
    if emit_c {
        use falcon_compiler::ir_to_c;
        let c_code = ir_to_c(&ir)
            .map_err(|e| anyhow::anyhow!("Code generation error: {}", e))?;
        let c_file = file.with_extension("c");
        std::fs::write(&c_file, &c_code)
            .map_err(|e| anyhow::anyhow!("Failed to write C file: {}", e))?;
        eprintln!("  ✓ Generated C code: {}", c_file.display());
        return Ok(());
    }

    #[cfg(not(feature = "llvm"))]
    let _ = (&output, opt, emit_llvm, run);
    
    // DEFAULT: Use LLVM backend to compile and run
    #[cfg(feature = "llvm")]
    {
        let module_name = file.file_stem().unwrap_or(std::ffi::OsStr::new("module")).to_string_lossy();
        let mut codegen = falcon_compiler::LlvmCodegen::new(&module_name);
        
        if let Err(e) = codegen.compile(&ir) {
            unsafe { codegen.shutdown(); }
            return Err(anyhow::anyhow!("LLVM compilation error: {}", e));
        }
        
        // Optimize
        if opt > 0 {
            eprintln!("  ⚙ Optimizing (O{})...", opt);
            if let Err(e) = codegen.optimize(opt) {
                unsafe { codegen.shutdown(); }
                return Err(anyhow::anyhow!("LLVM optimization error: {}", e));
            }
        }
        
        if run && profile_enum != Profile::Userland {
            unsafe { codegen.shutdown(); }
            let profile_name = match profile_enum {
                Profile::Kernel => "kernel",
                Profile::Baremetal => "baremetal",
                Profile::Userland => "userland",
            };
            let output_ext = match profile_enum {
                Profile::Kernel | Profile::Baremetal => "elf/.bin",
                Profile::Userland => "exe/out",
            };
            return Err(anyhow::anyhow!(
                "FREESTANDING PROFILE RUN ERROR:\n\
                 `--run` executes a hosted OS process and is only valid for userland.\n\
                 Requested profile: {}\n\
                 Kernel/Baremetal outputs are freestanding artifacts, not host processes.\n\
                 Next step: falcon build {} --profile {}\n\
                 Expected output: {}",
                profile_name,
                file.display(),
                profile_name,
                output_ext
            ));
        }

        // Determine output path
        let default_ext = match profile_enum {
            Profile::Userland => {
                #[cfg(windows)]
                {
                    "exe"
                }
                #[cfg(not(windows))]
                {
                    "out"
                }
            }
            Profile::Kernel | Profile::Baremetal => "elf",
        };

        let exe_file = output.unwrap_or_else(|| file.with_extension(default_ext));
        let obj_file = file.with_extension("o");
        
        // Emit object file — use CLI target override or profile default
        let effective_target = resolve_target_triple(profile_enum, &target);
        if effective_target.is_some() {
            eprintln!("  ⚙ Cross-compiling for target: {}", effective_target.as_ref().unwrap());
        }
        eprintln!("  ⚙ Generating native code...");
        if let Err(e) = codegen.write_object_file_for_triple(
            &obj_file,
            effective_target.as_deref(),
        ) {
            unsafe { codegen.shutdown(); }
            return Err(anyhow::anyhow!("Failed to emit object file: {}", e));
        }
        unsafe { codegen.shutdown(); }
        
        eprintln!("  Linking...");
        let output_dir = file.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();

        if profile_enum == Profile::Userland {
            // Resolve linker (FALCON_LINKER env, then platform defaults, then PATH)
            let linker = resolve_linker();
            let mut link_cmd = Command::new(&linker);

            // Hosted userland path links runtime C shim.
            let (runtime_c, runtime_include_dir) = setup_runtime(&output_dir, profile_enum)?;
            eprintln!("  Runtime: {}", runtime_c.display());
            link_cmd
                .arg(&obj_file)
                .arg(&runtime_c)
                .arg("-o")
                .arg(&exe_file)
                .arg("-I")
                .arg(&runtime_include_dir);
            #[cfg(windows)]
            {
                link_cmd.arg("-D_CRT_SECURE_NO_WARNINGS");
            }

            let status = link_cmd.status();
            match status {
                Ok(s) if s.success() => {
                    eprintln!("  Built: {}", exe_file.display());

                    // Clean up object file
                    let _ = std::fs::remove_file(&obj_file);

                    if run {
                        eprintln!("\nRunning...\n");
                        let run_path = if exe_file.is_absolute() {
                            exe_file.clone()
                        } else {
                            std::env::current_dir()?.join(&exe_file)
                        };
                        let run_status = Command::new(&run_path).status()?;
                        eprintln!("\nExit code: {}", run_status.code().unwrap_or(-1));
                    }
                }
                Ok(_) => {
                    return Err(anyhow::anyhow!(
                        "Linking failed for profile '{}'. Check linker/target/linker-script settings.",
                        profile
                    ));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Linker not found: {}. Install clang or set FALCON_LINKER.",
                        e
                    ));
                }
            }
        } else {
            // Freestanding kernel/baremetal path: no libc/runtime linkage.
            let freestanding_linker = resolve_freestanding_linker();
            let mut link_cmd = Command::new(&freestanding_linker);
            let script_path = setup_linker_script(&output_dir, profile_enum)?;
            let entry = freestanding_entry_symbol(profile_enum).unwrap_or("_start");
            eprintln!("  Freestanding linker: {}", freestanding_linker);
            eprintln!("  Linker script: {}", script_path.display());

            link_cmd
                .arg("-m")
                .arg("elf_x86_64")
                .arg("-T")
                .arg(&script_path)
                .arg("-e")
                .arg(entry)
                .arg("--gc-sections")
                .arg("-o")
                .arg(&exe_file)
                .arg(&obj_file);

            let status = link_cmd.status();
            match status {
                Ok(s) if s.success() => {
                    eprintln!("  Built: {}", exe_file.display());

                    if let Some(objcopy) = resolve_objcopy() {
                        let bin_file = exe_file.with_extension("bin");
                        let objcopy_status = Command::new(&objcopy)
                            .arg("-O")
                            .arg("binary")
                            .arg(&exe_file)
                            .arg(&bin_file)
                            .status();
                        match objcopy_status {
                            Ok(os) if os.success() => {
                                eprintln!("  Flat binary: {}", bin_file.display());
                            }
                            Ok(_) => {
                                eprintln!("  objcopy failed; .bin artifact was not produced");
                            }
                            Err(_) => {
                                eprintln!("  objcopy not available; skipping .bin artifact");
                            }
                        }
                    }

                    // Clean up object file
                    let _ = std::fs::remove_file(&obj_file);
                }
                Ok(_) => {
                    return Err(anyhow::anyhow!(
                        "Linking failed for profile '{}'. Check linker/target/linker-script settings.",
                        profile
                    ));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Freestanding linker not found: {}. Install lld or set FALCON_FREESTANDING_LINKER.",
                        e
                    ));
                }
            }
        }

        return Ok(());
    }
    
    #[cfg(not(feature = "llvm"))]
    {
        return Err(anyhow::anyhow!(
            "LLVM backend not enabled. Rebuild with: cargo build --features llvm"
        ));
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    // If a subcommand is provided, use it
    if let Some(command) = cli.command {
        match command {
            Commands::Build {
                file,
                profile,
                output,
                opt,
                emit_ir,
                emit_c,
                emit_llvm,
                sanitize: _sanitize,
                run,
                keep_generated,
                strict_imports,
                dump_imports,
                target,
            } => {
                build_falcon_program(
                    file,
                    profile,
                    output,
                    opt,
                    emit_ir,
                    emit_c,
                    emit_llvm,
                    run,
                    keep_generated,
                    strict_imports,
                    dump_imports,
                    target,
                )
            }
            
            Commands::Run {
                file,
                profile,
                jit,
                native,
                keep_generated,
            } => {
                let profile_enum = Profile::from_str(&profile)
                    .map_err(|e| anyhow::anyhow!("Invalid profile: {}", e))?;
                if profile_enum != Profile::Userland {
                    return Err(anyhow::anyhow!(
                        "FREESTANDING RUN MODE ERROR:\n\
                         `falcon run` is userland-only.\n\
                         Requested profile: {}\n\
                         Use: falcon build {} --profile {}",
                        profile,
                        file.display(),
                        profile
                    ));
                }

                #[cfg(windows)]
                let force_native = true;
                #[cfg(not(windows))]
                let force_native = false;

                #[cfg(windows)]
                if force_native {
                    if jit {
                        eprintln!("Warning: --jit is disabled on Windows due to an access-violation crash; using native mode.");
                    } else {
                        eprintln!("Warning: LLVM JIT is temporarily disabled on Windows due to an access-violation crash; using native mode.");
                    }
                }

                // Default to JIT if neither flag is specified
                if (native && !jit) || force_native {
                    // Use native compilation (compile + link + run)
                    eprintln!("🦅 Falcon run (native mode) - {}", file.display());
                    build_falcon_program(
                        file,
                        "userland".to_string(),
                        None,
                        2,
                        false,
                        false,
                        false,
                        true,
                        keep_generated,
                        false,
                        false,
                        None,
                    )
                } else {
                    // Default: Use JIT execution
                    eprintln!("🦅 Falcon run (JIT mode) - {}", file.display());
                    #[cfg(feature = "llvm")]
                    {
                        run_falcon_jit(file, keep_generated)
                    }
                    #[cfg(not(feature = "llvm"))]
                    {
                        Err(anyhow::anyhow!("JIT mode requires LLVM feature. Use 'falcon build' instead."))
                    }
                }
            }
            
            Commands::Check {
                file,
                profile,
                keep_generated,
                strict_imports,
                dump_imports,
            } => {
                use falcon_compiler::*;

                println!("Checking {} ({})", file.display(), profile);

                let profile_enum = Profile::from_str(&profile)
                    .map_err(|e| anyhow::anyhow!("Invalid profile: {}", e))?;

                let prepared = prepare_compilation_input(
                    &file,
                    profile_enum,
                    keep_generated,
                )?;
                let _cleanup = GeneratedFileCleanup::new(prepared.cleanup_generated.clone());
                let compile_file = prepared.compile_file;

                let source = std::fs::read_to_string(&compile_file)
                    .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

                let mut lexer = Lexer::new(&source);
                let (tokens, spans) = lexer.tokenize_with_spans()
                    .map_err(|e| anyhow::anyhow!("Lexer error: {}", e))?;

                let mut parser = Parser::new_with_spans(tokens, spans);
                let mut ast = parser.parse()
                    .map_err(|e| anyhow::anyhow!("Parser error: {}", e))?;

                passes::filter_ast_by_profile(&mut ast, profile_enum);

                let source_dir = compile_file.parent().unwrap_or(std::path::Path::new("."));
                let resolved_imports = passes::resolve_imports(&mut ast, source_dir, profile_enum)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                passes::validate_imports(&ast, profile_enum)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                passes::validate_trait_impls(&ast)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                // Monomorphize generics before IR lowering
                passes::monomorphize_generics(&mut ast);

                let mut ir = ir::ast_to_ir(&ast)
                    .map_err(|e| anyhow::anyhow!("IR generation error: {}", e))?;
                ir.imports = resolved_imports;

                if dump_imports {
                    if ir.imports.is_empty() {
                        println!("Resolved imports: (none)");
                    } else {
                        println!("Resolved imports:");
                        for import in &ir.imports {
                            println!("  {} -> {} @ {}", import.module, import.resolved_to, import.profile);
                        }
                    }
                }

                if strict_imports {
                    passes::lint_missing_library_imports(&ir)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                }

                passes::validate_ir_import_contract(&ir)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                passes::apply_profile_passes(&mut ir, profile_enum)
                    .map_err(|e| anyhow::anyhow!("Profile pass error: {}", e))?;

                passes::verify_ownership(&ir)
                    .map_err(|e| anyhow::anyhow!("Ownership verification error: {}", e))?;

                passes::validate_function_calls(&ir)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                println!("Check passed");
                Ok(())
            }

            Commands::Fmt { files, check } => {
                if check {
                    println!("Checking formatting of {} file(s)", files.len());
                } else {
                    println!("Formatting {} file(s)", files.len());
                }
                // TODO: Implement formatting
                Ok(())
            }
        }
    } else if let Some(file) = cli.file {
        // Direct file argument: falcon example.fc
        
        // Check if multi-profile build requested
        if let Some(profiles_arg) = cli.profiles {
            let profiles_list: Vec<&str> = if profiles_arg == "all" {
                vec!["userland", "kernel", "baremetal"]
            } else {
                profiles_arg.split(',').map(|s| s.trim()).collect()
            };
            
            eprintln!("🦅 Falcon - multi-profile build for {} with profiles: {:?}", file.display(), profiles_list);
            
            let mut all_ok = true;
            for profile in profiles_list {
                eprintln!("\n=== Building with {} profile ===", profile);
                
                // Generate profile-specific output filename
                let base_name = file.file_stem().unwrap_or(std::ffi::OsStr::new("out")).to_string_lossy();
                let output_ext = match profile {
                    "userland" => {
                        #[cfg(windows)]
                        {
                            "exe"
                        }
                        #[cfg(not(windows))]
                        {
                            "out"
                        }
                    }
                    "kernel" | "baremetal" => "elf",
                    _ => "out",
                };
                let output_file = file
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join(format!("{}.{}.{}", base_name, profile, output_ext));
                
                match build_falcon_program(
                    file.clone(),
                    profile.to_string(),
                    Some(output_file),
                    0,
                    false,
                    false,
                    false,
                    false,
                    cli.keep_generated,
                    false,
                    false,
                    None,
                ) {
                    Ok(_) => eprintln!("  ✓ {} profile: OK", profile),
                    Err(e) => {
                        eprintln!("  ✗ {} profile: {}", profile, e);
                        all_ok = false;
                    }
                }
            }
            
            if all_ok {
                eprintln!("\n✓ All profiles built successfully!");
                Ok(())
            } else {
                Err(anyhow::anyhow!("Some profiles failed to build"))
            }
        } else {
            // Single profile (default: userland)
            eprintln!("🦅 Falcon - running {} with {} profile", file.display(), cli.profile);
            build_falcon_program(
                file,
                cli.profile,
                None,
                0,
                false,
                false,
                false,
                true,
                cli.keep_generated,
                false,
                false,
                None,
            )
        }
    } else {
        // No subcommand and no file - show help
        Err(anyhow::anyhow!("Usage: falcon <file.fc> or falcon <command>\n\nFor help: falcon --help"))
    }
}

#[cfg(test)]
mod tests {
    use super::{generated_fc_path, is_fpy_source, runtime_source_filename};
    use falcon_compiler::Profile;
    use std::path::PathBuf;

    #[test]
    fn runtime_source_selection_matches_profile() {
        assert_eq!(runtime_source_filename(Profile::Userland), "falcon_runtime.c");
        assert_eq!(runtime_source_filename(Profile::Kernel), "falcon_runtime_kernel.c");
        assert_eq!(
            runtime_source_filename(Profile::Baremetal),
            "falcon_runtime_baremetal.c"
        );
    }

    #[test]
    fn python_style_file_detection_and_generated_name() {
        let file = PathBuf::from("script.fpy");
        assert!(is_fpy_source(&file));
        assert_eq!(
            generated_fc_path(&file).unwrap(),
            PathBuf::from("script.__gen__.fc")
        );
        assert!(!is_fpy_source(&PathBuf::from("script.fc")));
    }
}


