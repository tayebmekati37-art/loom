cargo : warning: unused imports: `Condition`, `Literal`, `Source`, and `Statement`
At line:129 char:15
+     $result = cargo run -- translate -f $input -l cobol -t rust 2>&1
+               ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: unused...and `Statement`:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError
 
 --> src\translate_python.rs:1:27
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition};
  |                           ^^^^^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused imports: `Condition`, `Literal`, `Source`, and `Statement`
 --> src\translate_javascript.rs:1:27
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition};
  |                           ^^^^^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^^^^

warning: unused imports: `Condition`, `Literal`, `Source`, and `Statement`
 --> src\translate_csharp.rs:1:27
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition};
  |                           ^^^^^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^^^^

warning: unused imports: `Condition`, `Literal`, `Source`, and `Statement`
 --> src\translate_go.rs:1:27
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition};
  |                           ^^^^^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^^^^

warning: unused imports: `Condition` and `StringSource`
 --> src\translate_rust.rs:1:55
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition, FileMode, WhenCondition, LiteralOrVariable, StringSource};
  |                                                       ^^^^^^^^^                                              ^^^^^^^^^^^^

warning: unused imports: `Condition`, `Literal`, `Source`, and `Statement`
 --> src\translate_typescript.rs:1:27
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition};
  |                           ^^^^^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^^^^

warning: unused imports: `Condition`, `Literal`, `Source`, and `Statement`
 --> src\translate_kotlin.rs:1:27
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition};
  |                           ^^^^^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^^^^

warning: unused imports: `Condition`, `Literal`, `Source`, and `Statement`
 --> src\translate_swift.rs:1:27
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition};
  |                           ^^^^^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^^^^

warning: unused imports: `Condition`, `Literal`, `Source`, and `Statement`
 --> src\translate_zig.rs:1:27
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition};
  |                           ^^^^^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^^^^

warning: unused imports: `Condition`, `Literal`, `Source`, and `Statement`
 --> src\translate_nim.rs:1:27
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition};
  |                           ^^^^^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^^^^

warning: unused imports: `Condition`, `Literal`, `Source`, and `Statement`
 --> src\translate_dart.rs:1:27
  |
1 | use crate::ir::{Function, Statement, Source, Literal, Condition};
  |                           ^^^^^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^^^^

warning: unused variable: `value`
  --> src\parser_pli.rs:23:28
   |
23 |                 if let (Ok(value), Ok(_)) = (left_op.parse::<i64>(), right_op.parse::<i64>()) {
   |                            ^^^^^ help: if this is intentional, prefix it with an underscore: `_value`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `function`
 --> src\translate_python.rs:4:18
  |
4 | pub fn translate(function: &Function) -> String {
  |                  ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_function`

warning: unused variable: `function`
 --> src\translate_javascript.rs:4:18
  |
4 | pub fn translate(function: &Function) -> String {
  |                  ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_function`

warning: unused variable: `function`
 --> src\translate_csharp.rs:4:18
  |
4 | pub fn translate(function: &Function) -> String {
  |                  ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_function`

warning: unused variable: `function`
 --> src\translate_go.rs:4:18
  |
4 | pub fn translate(function: &Function) -> String {
  |                  ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_function`

warning: unreachable pattern
   --> src\translate_rust.rs:192:9
    |
192 |         _ => {
    |         ^ no value can reach this
    |
note: multiple earlier patterns match some of the same values
   --> src\translate_rust.rs:192:9
    |
 21 |         Statement::Add { target, value } => {
    |         -------------------------------- matches some of the same values
...
 24 |         Statement::Move { source, target } => {
    |         ---------------------------------- matches some of the same values
...
 32 |         Statement::If { condition, then_branch, else_branch } => {
    |         ----------------------------------------------------- matches some of the same values
...
 46 |         Statement::Perform { name } => {
    |         --------------------------- matches some of the same values
...
192 |         _ => {
    |         ^ ...and 20 other patterns collectively make this unreachable
    = note: `#[warn(unreachable_patterns)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `function`
 --> src\translate_typescript.rs:4:18
  |
4 | pub fn translate(function: &Function) -> String {
  |                  ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_function`

warning: unused variable: `function`
 --> src\translate_kotlin.rs:4:18
  |
4 | pub fn translate(function: &Function) -> String {
  |                  ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_function`

warning: unused variable: `function`
 --> src\translate_swift.rs:4:18
  |
4 | pub fn translate(function: &Function) -> String {
  |                  ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_function`

warning: unused variable: `function`
 --> src\translate_zig.rs:4:18
  |
4 | pub fn translate(function: &Function) -> String {
  |                  ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_function`

warning: unused variable: `function`
 --> src\translate_nim.rs:4:18
  |
4 | pub fn translate(function: &Function) -> String {
  |                  ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_function`

warning: unused variable: `function`
 --> src\translate_dart.rs:4:18
  |
4 | pub fn translate(function: &Function) -> String {
  |                  ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_function`

warning: trait `LegacyRunner` is never used
 --> src\interpreter.rs:5:11
  |
5 | pub trait LegacyRunner {
  |           ^^^^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: struct `CommandRunner` is never constructed
  --> src\interpreter.rs:13:12
   |
13 | pub struct CommandRunner {
   |            ^^^^^^^^^^^^^

warning: associated function `new` is never used
  --> src\interpreter.rs:20:12
   |
19 | impl CommandRunner {
   | ------------------ associated function in this implementation
20 |     pub fn new(command: &str, args: Vec<String>, ext: &str) -> Self {
   |            ^^^

warning: function `parse_output` is never used
  --> src\interpreter.rs:65:4
   |
65 | fn parse_output(s: &str) -> anyhow::Result<HashMap<String, i64>> {
   |    ^^^^^^^^^^^^

warning: variant `Mixed` is never constructed
  --> src\migration.rs:14:5
   |
11 | pub enum Routing {
   |          ------- variant in this enum
...
14 |     Mixed,
   |     ^^^^^
   |
   = note: `Routing` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead code analysis

warning: `loom` (bin "loom") generated 28 warnings (run `cargo fix --bin "loom" -p loom` to apply 22 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running `target\debug\loom.exe translate -f C:\Users\Tayeb\Documents\loom\tests_cobol\move.cob -l cobol -t rust`
fn translated_func() -> Result<(), Box<dyn std::error::Error>> {
    B. = A;
    println!("B.");
    return Ok(());
    Ok(())
}

