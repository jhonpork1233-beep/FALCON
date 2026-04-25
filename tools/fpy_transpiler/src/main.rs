use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "falcon-fpy")]
#[command(about = "Transpile Python-style Falcon (.fpy) to Falcon (.fc)")]
struct Cli {
    /// Input .fpy file
    #[arg(required = true)]
    input: PathBuf,

    /// Output .fc file (default: <input_stem>.__gen__.fc)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let source = std::fs::read_to_string(&cli.input)
        .map_err(|e| format!("Failed to read {}: {}", cli.input.display(), e))?;

    let transpiled =
        falcon_fpy_transpiler::transpile_source(&source, &cli.input.display().to_string())?;

    let output = cli.output.unwrap_or_else(|| {
        let stem = cli
            .input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        cli.input
            .with_file_name(format!("{}.__gen__.fc", stem))
    });

    std::fs::write(&output, transpiled)
        .map_err(|e| format!("Failed to write {}: {}", output.display(), e))?;

    eprintln!("Transpiled {} -> {}", cli.input.display(), output.display());
    Ok(())
}
