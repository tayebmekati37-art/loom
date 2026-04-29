mod parser;
mod parser_cobol;
mod parser_rpg;
mod parser_pli;
mod parser_jcl;
mod parser_asm;
mod ir;
mod translate_python;
mod translate_javascript;
mod translate_csharp;
mod translate_go;
mod translate_rust;
mod translate_typescript;
mod translate_kotlin;
mod translate_swift;
mod translate_zig;
mod translate_nim;
mod translate_dart;
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
    Translate {
        #[arg(short = 'f', long)]
        input: String,
        #[arg(short = 'l', long, default_value = "simple")]
        lang: String,
        #[arg(short = 't', long, default_value = "python")]
        target: String,
    },
    Validate {
        #[arg(short = 'f', long)]
        input: String,
        #[arg(short = 'l', long, default_value = "simple")]
        lang: String,
        #[arg(short = 'v', long, value_parser = parse_key_val)]
        inputs: Vec<(String, i64)>,
        #[arg(short = 'r', long)]
        record: bool,
        #[arg(short = 'c', long)]
        test_file: Option<String>,
    },
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
        Commands::Translate { input, lang, target } => {
            let bytes = std::fs::read(&input)?;
            let content = String::from_utf8_lossy(&bytes).to_string();
            let content = content.trim_start_matches('\u{feff}').to_string();
            let statements = match lang.as_str() {
                "simple" => parser::parse_program(&content)?,
                "cobol" => parser_cobol::parse_program(&content)?,
                "rpg" => parser_rpg::parse_program(&content)?,
                "pli" => parser_pli::parse_program(&content)?,
                "jcl" => parser_jcl::parse_program(&content)?,
                "asm" => parser_asm::parse_program(&content)?,
                _ => anyhow::bail!("Unsupported legacy language: {}", lang),
            };
            let func = ir::Function {
                name: "translated_func".to_string(),
                body: statements,
            };
            match target.as_str() {
                "python" => println!("{}", translate_python::translate(&func)),
                "javascript" => println!("{}", translate_javascript::translate(&func)),
                "csharp" => println!("{}", translate_csharp::translate(&func)),
                "go" => println!("{}", translate_go::translate(&func)),
                "rust" => println!("{}", translate_rust::translate(&func)),
                "typescript" => println!("{}", translate_typescript::translate(&func)),
                "kotlin" => println!("{}", translate_kotlin::translate(&func)),
                "swift" => println!("{}", translate_swift::translate(&func)),
                "zig" => println!("{}", translate_zig::translate(&func)),
                "nim" => println!("{}", translate_nim::translate(&func)),
                "dart" => println!("{}", translate_dart::translate(&func)),
                _ => anyhow::bail!("Unsupported target: {}", target),
            }
        }
        Commands::Validate { input, lang, inputs, record, test_file } => {
            let bytes = std::fs::read(&input)?;
            let content = String::from_utf8_lossy(&bytes).to_string();
            let content = content.trim_start_matches('\u{feff}').to_string();
            let statements = match lang.as_str() {
                "simple" => parser::parse_program(&content)?,
                "cobol" => parser_cobol::parse_program(&content)?,
                "rpg" => parser_rpg::parse_program(&content)?,
                "pli" => parser_pli::parse_program(&content)?,
                "jcl" => parser_jcl::parse_program(&content)?,
                "asm" => parser_asm::parse_program(&content)?,
                _ => anyhow::bail!("Unsupported legacy language: {}", lang),
            };
            let func = ir::Function {
                name: "translated_func".to_string(),
                body: statements,
            };
            let test_path = test_file.unwrap_or_else(|| format!("{}.tests.json", input));
            let test_cases = if record {
                let cases = generate_test_cases(&func)?;
                let json_cases: Vec<serde_json::Value> = cases.iter().map(|map| {
                    let mut obj = serde_json::Map::new();
                    for (k, v) in map {
                        obj.insert(k.clone(), serde_json::json!(v));
                    }
                    serde_json::Value::Object(obj)
                }).collect();
                std::fs::write(&test_path, serde_json::to_string_pretty(&json_cases)?)?;
                println!("Recorded {} test cases to {}", cases.len(), test_path);
                cases
            } else if !inputs.is_empty() {
                vec![inputs.into_iter().collect()]
            } else {
                if std::path::Path::new(&test_path).exists() {
                    let data = std::fs::read_to_string(&test_path)?;
                    let json_cases: Vec<serde_json::Map<String, serde_json::Value>> = serde_json::from_str(&data)?;
                    let mut cases = Vec::new();
                    for map in json_cases {
                        let mut hm = HashMap::new();
                        for (k, v) in map {
                            if let Some(num) = v.as_i64() {
                                hm.insert(k, num);
                            }
                        }
                        cases.push(hm);
                    }
                    cases
                } else {
                    generate_test_cases(&func)?
                }
            };
            let mut passed = true;
            for (i, inputs_map) in test_cases.iter().enumerate() {
                let mut interpreter = interpreter::Interpreter::new();
                interpreter.add_function(func.clone());
                let legacy_output = interpreter.run(&func.name, inputs_map.clone());
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
            let bytes = std::fs::read(&legacy_file)?;
            let content = String::from_utf8_lossy(&bytes).to_string();
            let content = content.trim_start_matches('\u{feff}').to_string();
            let legacy_func = ir::Function {
                name: "legacy_func".to_string(),
                body: parser::parse_program(&content)?,
            };
            let mut fig = migration::StranglerFig::new();
            fig.add_legacy(legacy_func.name.clone(), legacy_func.clone());
            if let Some(modern_path) = modern_file {
                let bytes = std::fs::read(&modern_path)?;
                let content = String::from_utf8_lossy(&bytes).to_string();
                let content = content.trim_start_matches('\u{feff}').to_string();
                let modern_func = ir::Function {
                    name: "modern_func".to_string(),
                    body: parser::parse_program(&content)?,
                };
                fig.add_modern(modern_func.name.clone(), modern_func);
            }
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
    for name in inputs.keys() {
        writeln!(temp_file, "    print('{}={{}}'.format({}))", name, name)?;
    }
    let path = temp_file.path().to_str().unwrap();
    let output = Command::new("python")
        .arg(path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        anyhow::bail!("Python execution failed: {}", stderr);
    }
    let stdout = String::from_utf8(output.stdout)?;
    parse_output(&stdout)
}

fn parse_output(s: &str) -> anyhow::Result<HashMap<String, i64>> {
    let mut map = HashMap::new();
    for line in s.lines() {
        if let Some((key, val)) = line.split_once('=') {
            map.insert(key.trim().to_string(), val.trim().parse::<i64>()?);
        }
    }
    Ok(map)
}

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
            ir::Statement::Add { target, .. } => { set.insert(target.clone()); }
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
            ir::Statement::Display { .. } => {}
            ir::Statement::Evaluate { subject, also_subject, when_clauses } => {
                set.insert(subject.clone());
                if let Some(also) = also_subject {
                    set.insert(also.clone());
                }
                for when in when_clauses {
                    if let ir::WhenCondition::Variable(v) = &when.condition {
                        set.insert(v.clone());
                    }
                    collect_variables(&when.body, set);
                }
            }
            ir::Statement::String { sources, into, .. } => {
                set.insert(into.clone());
                for src in sources {
                    if let ir::LiteralOrVariable::Variable(v) = &src.source {
                        set.insert(v.clone());
                    }
                }
            }
            ir::Statement::Unstring { source, into, .. } => {
                set.insert(source.clone());
                for var in into {
                    set.insert(var.clone());
                }
            }
            ir::Statement::OpenFile { .. } => {}
            ir::Statement::ReadFile { .. } => {}
            ir::Statement::WriteFile { .. } => {}
            ir::Statement::CloseFile { .. } => {}
        }
    }
}