mod parser;
mod ir;
mod translate_python;
mod translate_javascript;
mod interpreter;
mod migration;

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
    /// Translate a legacy file to a target language
    Translate {
        #[arg(short = 'f', long)]
        input: String,
        #[arg(short = 't', long, default_value = "python")]
        target: String,
    },
    /// Validate translation by comparing legacy interpreter output with Python output
    Validate {
        #[arg(short = 'f', long)]
        input: String,
        /// Input values as key=value pairs, e.g., x=5 y=10
        #[arg(short = 'v', long, value_parser = parse_key_val)]
        inputs: Vec<(String, i64)>,
    },
    /// Generate wrapper for incremental migration (strangler fig)
    Migrate {
        #[arg(short = 'l', long)]
        legacy_file: String,
        #[arg(short = 'm', long)]
        modern_file: Option<String>,
        #[arg(short = 't', long, default_value = "python")]
        target: String,
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
        Commands::Translate { input, target } => {
            let content = std::fs::read_to_string(&input)?;
            let statements = parser::parse_program(&content)?;
            let func = ir::Function {
                name: "translated_func".to_string(),
                body: statements,
            };
            match target.as_str() {
                "python" => {
                    let code = translate_python::translate(&func);
                    println!("{}", code);
                }
                "javascript" => {
                    let code = translate_javascript::translate(&func);
                    println!("{}", code);
                }
                _ => anyhow::bail!("Unsupported target: {}", target),
            }
        }
        Commands::Validate { input, inputs } => {
            let content = std::fs::read_to_string(&input)?;
            let statements = parser::parse_program(&content)?;
            let func = ir::Function {
                name: "translated_func".to_string(),
                body: statements,
            };

            // Generate test cases (if no explicit inputs provided)
            let test_cases = if inputs.is_empty() {
                generate_test_cases(&func)?
            } else {
                vec![inputs.into_iter().collect()]
            };

            let mut passed = true;
            for (i, inputs_map) in test_cases.iter().enumerate() {
                // Run legacy interpreter
                let mut interpreter = interpreter::Interpreter::new();
                interpreter.add_function(func.clone());
                let legacy_output = interpreter.run(&func.name, inputs_map.clone());

                // Generate Python code and run it
                let python_code = translate_python::translate(&func);
                let python_output = run_python(&python_code, inputs_map)?;

                if legacy_output == python_output {
                    println!("Test case {} PASSED", i);
                } else {
                    passed = false;
                    println!("Test case {} FAILED", i);
                    println!("  Inputs: {:?}", inputs_map);
                    println!("  Legacy output: {:?}", legacy_output);
                    println!("  Python output: {:?}", python_output);
                }
            }
            if passed {
                println!("All test cases passed.");
            } else {
                anyhow::bail!("Validation failed");
            }
        }
        Commands::Migrate { legacy_file, modern_file, target } => {
            let legacy_code = std::fs::read_to_string(&legacy_file)?;
            let legacy_func = ir::Function {
                name: "legacy_func".to_string(),
                body: parser::parse_program(&legacy_code)?,
            };
            let mut fig = migration::StranglerFig::new();
            fig.add_legacy(legacy_func.name.clone(), legacy_func.clone());

            if let Some(modern_path) = modern_file {
                let modern_code = std::fs::read_to_string(modern_path)?;
                let modern_func = ir::Function {
                    name: "modern_func".to_string(),
                    body: parser::parse_program(&modern_code)?,
                };
                fig.add_modern(modern_func.name.clone(), modern_func);
            }

            // Example: route the legacy function to modern
            fig.set_routing("legacy_func", migration::Routing::Modern);

            let wrapper = fig.generate_wrapper_code(&target);
            println!("{}", wrapper);
        }
    }
    Ok(())
}

// --- Helper functions for validation ---

fn run_python(code: &str, inputs: &HashMap<String, i64>) -> anyhow::Result<HashMap<String, i64>> {
    use std::process::Command;
    use tempfile::NamedTempFile;

    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", code)?;
    writeln!(temp_file, "if __name__ == '__main__':")?;
    for (name, value) in inputs {
        writeln!(temp_file, "    {} = {}", name, value)?;
    }
    writeln!(temp_file, "    translated_func()")?;
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
    parse_python_dict(&stdout)
}

fn parse_python_dict(s: &str) -> anyhow::Result<HashMap<String, i64>> {
    let s = s.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        anyhow::bail!("Invalid dict format: {}", s);
    }
    let inner = &s[1..s.len() - 1];
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

// --- Test case generation ---

use rand::Rng;

fn generate_test_cases(func: &ir::Function) -> anyhow::Result<Vec<HashMap<String, i64>>> {
    let mut vars = std::collections::HashSet::new();
    collect_variables(&func.body, &mut vars);
    if vars.is_empty() {
        return Ok(vec![]);
    }
    let mut tests = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..5 {
        let mut map = HashMap::new();
        for var in &vars {
            let val = rng.gen_range(-100..100);
            map.insert(var.clone(), val);
        }
        tests.push(map);
    }
    Ok(tests)
}

fn collect_variables(stmts: &[ir::Statement], set: &mut std::collections::HashSet<String>) {
    for stmt in stmts {
        match stmt {
            ir::Statement::Add { target, .. } => {
                set.insert(target.clone());
            }
            ir::Statement::Move { source, target } => {
                set.insert(target.clone());
                if let ir::Source::Variable(v) = source {
                    set.insert(v.clone());
                }
            }
            ir::Statement::If { condition, then_branch, else_branch } => {
                set.insert(condition.left.clone());
                collect_variables(then_branch, set);
                if let Some(b) = else_branch {
                    collect_variables(b, set);
                }
            }
            ir::Statement::Perform { .. } => {}
            ir::Statement::While { condition, body } => {
                set.insert(condition.left.clone());
                collect_variables(body, set);
            }
        }
    }
}