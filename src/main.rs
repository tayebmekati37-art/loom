mod parser;
mod ir;
mod translate_python;

use clap::{Parser as ClapParser, Subcommand};

#[derive(ClapParser)]
#[command(name = "loom")]
#[command(about = "Legacy code modernization tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Translate a legacy file to Python
    Translate {
        /// Input file with legacy code
        #[arg(short, long)]
        input: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
    Commands::Translate { input } => {
        let content = std::fs::read_to_string(input)?;
        println!("Raw content: {:?}", content);  // Debug
        let statements = parser::parse_program(&content)?;
        println!("Parsed {} statements", statements.len()); // Debug
        for (i, stmt) in statements.iter().enumerate() {
            println!("Statement {}: {:?}", i, stmt);
        }
        let func = ir::Function {
            name: "translated_func".to_string(),
            body: statements,
        };
        let python_code = translate_python::translate(&func);
        println!("{}", python_code);
    }
}
    Ok(())
}