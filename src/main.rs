mod parser;
mod ir;
mod translate_python;
mod interpreter;

use clap::{Parser as ClapParser, Subcommand};
use std::collections::HashMap;
use std::io::Write;

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
        #[arg(short = 'f', long)]
        input: String,
    },
    /// Validate translation by comparing legacy interpreter output with Python output
    Validate {
        #[arg(short = 'f', long)]
        input: String,
        /// Input values as key=value pairs, e.g., x=5 y=10
        #[arg(short = 'v', long, value_parser = parse_key_val)]
        inputs: Vec<(String, i64)>,
    },
}

fn parse_key_val(s: &str) -> Result<(String, i64), String> {
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() != 2 {
        return Err("Format must be key=value".to_string());
    }
    let value = parts[1].parse::<i64>().map_err(|_| "Value must be an integer")?;
    Ok((parts[0].to_string(), value))
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Translate { input } => {
            let content = std::fs::read_to_string(&input)?;
            let statements = parser::parse_program(&content)?;
            let func = ir::Function {
                name: "translated_func".to_string(),
                body: statements,
            };
            let python_code = translate_python::translate(&func);
            println!("{}", python_code);
        }
        Commands::Validate { input, inputs } => {
            let content = std::fs::read_to_string(&input)?;
            let statements = parser::parse_program(&content)?;
            let func = ir::Function {
                name: "translated_func".to_string(),
                body: statements,
            };

            // Run legacy interpreter
            let mut interpreter = interpreter::Interpreter::new();
            let inputs_map: HashMap<_, _> = inputs.clone().into_iter().collect();
            let legacy_output = interpreter.run(&func, inputs_map.clone());

            // Generate Python code and run it
            let python_code = translate_python::translate(&func);
            let python_output = run_python(&python_code, &inputs_map)?;

            // Compare outputs
            if legacy_output == python_output {
                println!("Validation PASSED");
                println!("Legacy output: {:?}", legacy_output);
                println!("Python output: {:?}", python_output);
            } else {
                println!("Validation FAILED");
                println!("Legacy output: {:?}", legacy_output);
                println!("Python output: {:?}", python_output);
            }
        }
    }
    Ok(())
}

fn run_python(code: &str, inputs: &HashMap<String, i64>) -> anyhow::Result<HashMap<String, i64>> {
    use std::process::Command;
    use tempfile::NamedTempFile;

    // Create a temporary Python file
    let mut temp_file = NamedTempFile::new()?;
    // Write the function definition
    writeln!(temp_file, "{}", code)?;
    // Write a harness that reads inputs, calls the function, and prints final variable values
    writeln!(temp_file, "if __name__ == '__main__':")?;
    for (name, value) in inputs {
        writeln!(temp_file, "    {} = {}", name, value)?;
    }
    writeln!(temp_file, "    translated_func()")?;
    // Print the dictionary of variable values (all variables that were in inputs)
    writeln!(temp_file, "    result = {{}}")?;
    for name in inputs.keys() {
        writeln!(temp_file, "    result['{}'] = {}", name, name)?;
    }
    writeln!(temp_file, "    print(result)")?;

    let path = temp_file.path().to_str().unwrap();
    let output = Command::new("python")
        .arg(path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        anyhow::bail!("Python execution failed: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let result = parse_python_dict(&stdout)?;
    Ok(result)
}

fn parse_python_dict(s: &str) -> anyhow::Result<HashMap<String, i64>> {
    // Very simple parser: expects something like "{'x': 5, 'y': 10}"
    let s = s.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        anyhow::bail!("Invalid dict format: {}", s);
    }
    let inner = &s[1..s.len()-1];
    let mut map = HashMap::new();
    if inner.is_empty() {
        return Ok(map);
    }
    for pair in inner.split(',') {
        let parts: Vec<&str> = pair.split(':').collect();
        if parts.len() != 2 {
            continue;
        }
        let key = parts[0].trim().trim_matches('\'').to_string();
        let value = parts[1].trim().parse::<i64>()?;
        map.insert(key, value);
    }
    Ok(map)
}