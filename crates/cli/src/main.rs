#![forbid(unsafe_code)]

use frontend::ast::*;
use frontend::parser::Parser;
use frontend::typecheck::TypeChecker;
use interp::{Interpreter, Value};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
enum CliError {
    #[error("{0}")]
    Message(String),
}

fn main() -> Result<(), CliError> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        eprintln!("usage: gaut <file.gaut>");
        std::process::exit(1);
    }

    let file = PathBuf::from(args.remove(0));
    let std_dir = env::var("GAUT_STD_DIR").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("std"));
    let program = load_with_imports(&file, &std_dir)?;

    // builtin stubs for typechecker
    let mut decls = program.decls;
    append_builtin_prints(&mut decls);
    let program = Program { decls };

    let mut tc = TypeChecker::new();
    tc.check_program(&program).map_err(|e| CliError::Message(format!("type error: {e}")))?;

    let mut interp = Interpreter::new(1024 * 1024);
    interp.load_program(&program).map_err(|e| CliError::Message(format!("interp load error: {e}")))?;
    let result = interp.run_main().map_err(|e| CliError::Message(format!("runtime error: {e}")))?;
    println!("{result:?}");
    Ok(())
}

fn load_with_imports(entry: &Path, std_dir: &Path) -> Result<Program, CliError> {
    let mut visited = HashSet::new();
    let mut decls = Vec::new();
    load_recursive(entry, std_dir, &mut visited, &mut decls)?;
    Ok(Program { decls })
}

fn load_recursive(path: &Path, std_dir: &Path, visited: &mut HashSet<PathBuf>, out: &mut Vec<Decl>) -> Result<(), CliError> {
    let path = path
        .canonicalize()
        .map_err(|_| CliError::Message(format!("cannot canonicalize {}", path.display())))?;
    if !visited.insert(path.clone()) {
        return Ok(());
    }
    let src = fs::read_to_string(&path)
        .map_err(|_| CliError::Message(format!("failed to read {}", path.display())))?;
    let mut parser = Parser::new(&src)
        .map_err(|e| CliError::Message(format!("parse error in {}: {e}", path.display())))?;
    let program = parser
        .parse_program()
        .map_err(|e| CliError::Message(format!("parse error in {}: {e}", path.display())))?;

    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    // process imports first
    for decl in &program.decls {
        if let Decl::Import(imp) = decl {
            let mod_name = imp.module.0.clone();
            let local_path = base_dir.join(format!("{}.gaut", mod_name));
            let std_path = std_dir.join(format!("{}.gaut", mod_name));
            let target = if local_path.exists() {
                local_path
            } else if std_path.exists() {
                std_path
            } else {
                return Err(CliError::Message(format!(
                    "module '{}' not found in {} or {}",
                    mod_name,
                    base_dir.display(),
                    std_dir.display()
                )));
            };
            load_recursive(&target, std_dir, visited, out)?;
        }
    }

    out.extend(program.decls.into_iter());
    Ok(())
}

fn append_builtin_prints(decls: &mut Vec<Decl>) {
    let names: HashSet<_> = decls
        .iter()
        .filter_map(|d| match d {
            Decl::Func(f) => Some(f.name.0.clone()),
            _ => None,
        })
        .collect();
    let print_param = Param { mutable: false, name: Ident("msg".into()), ty: Type::Named(Ident("Str".into())) };
    if !names.contains("print") {
        decls.push(Decl::Func(FuncDecl {
            name: Ident("print".into()),
            params: vec![print_param.clone()],
            ret: Some(Type::Named(Ident("Str".into()))),
            body: Expr::Path(Path(vec![Ident("msg".into())])),
        }));
    }
    if !names.contains("println") {
        decls.push(Decl::Func(FuncDecl {
            name: Ident("println".into()),
            params: vec![print_param],
            ret: Some(Type::Named(Ident("Str".into()))),
            body: Expr::Path(Path(vec![Ident("msg".into())])),
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_calc() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo = manifest.parent().unwrap().parent().unwrap().to_path_buf();
        let root = repo.join("examples/calc.gaut");
        let std_dir = repo.join("std");
        let program = load_with_imports(&root, &std_dir).unwrap();
        let mut tc = TypeChecker::new();
        tc.check_program(&program).unwrap();
        let mut interp = Interpreter::new(1024 * 1024);
        interp.load_program(&program).unwrap();
        let v = interp.run_main().unwrap();
        assert_eq!(v, Value::Int(30));
    }
}
