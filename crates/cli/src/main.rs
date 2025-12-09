#![forbid(unsafe_code)]

use cgen::generate_c;
use frontend::ast::*;
use frontend::parser::Parser;
use frontend::typecheck::TypeChecker;
use interp::Interpreter;
#[cfg(test)]
use interp::Value;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
enum CliError {
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone)]
enum Mode {
    Run {
        file: PathBuf,
    },
    Emit {
        file: PathBuf,
        emit_c: PathBuf,
        build: Option<PathBuf>,
    },
}

fn main() -> Result<(), CliError> {
    let mode = parse_args(env::args().skip(1).collect())?;

    match mode {
        Mode::Run { file } => run_interpreter(&file),
        Mode::Emit {
            file,
            emit_c,
            build,
        } => emit_and_maybe_build(&file, &emit_c, build.as_ref()),
    }
}

fn parse_args(args: Vec<String>) -> Result<Mode, CliError> {
    if args.is_empty() {
        eprintln!("usage: gaut [--emit-c out.c] [--build out_bin] <file.gaut>");
        std::process::exit(1);
    }
    let mut emit_c = None;
    let mut build = None;
    let mut file = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--emit-c" => {
                let path = iter
                    .next()
                    .ok_or_else(|| CliError::Message("expected path after --emit-c".into()))?;
                emit_c = Some(PathBuf::from(path));
            }
            "--build" => {
                let path = iter.next().ok_or_else(|| {
                    CliError::Message("expected binary path after --build".into())
                })?;
                build = Some(PathBuf::from(path));
            }
            other if file.is_none() => {
                file = Some(PathBuf::from(other));
            }
            _ => return Err(CliError::Message("unexpected arguments".into())),
        }
    }

    let file = file.ok_or_else(|| CliError::Message("no input file provided".into()))?;
    if emit_c.is_none() && build.is_some() {
        emit_c = Some(PathBuf::from("target/gaut_out.c"));
    }

    if let Some(out) = emit_c {
        Ok(Mode::Emit {
            file,
            emit_c: out,
            build,
        })
    } else {
        Ok(Mode::Run { file })
    }
}

fn run_interpreter(file: &Path) -> Result<(), CliError> {
    let std_dir = std_dir();
    let program = load_with_imports(file, &std_dir)?;

    let mut decls = program.decls;
    append_builtin_prints(&mut decls);
    let program = Program { decls };

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .map_err(|e| CliError::Message(format!("type error: {e}")))?;

    let mut interp = Interpreter::new(1024 * 1024);
    interp
        .load_program(&program)
        .map_err(|e| CliError::Message(format!("interp load error: {e}")))?;
    let result = interp
        .run_main()
        .map_err(|e| CliError::Message(format!("runtime error: {e}")))?;
    println!("{result:?}");
    Ok(())
}

fn emit_and_maybe_build(
    file: &Path,
    c_out: &Path,
    build: Option<&PathBuf>,
) -> Result<(), CliError> {
    let std_dir = std_dir();
    let program = load_with_imports(file, &std_dir)?;
    let mut decls = program.decls;
    append_builtin_prints(&mut decls);
    let program = Program { decls };

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .map_err(|e| CliError::Message(format!("type error: {e}")))?;

    let c_src = generate_c(&program).map_err(|e| CliError::Message(format!("cgen error: {e}")))?;
    if let Some(parent) = c_out.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CliError::Message(format!("create dir {}: {e}", parent.display())))?;
    }
    let mut f = fs::File::create(c_out)
        .map_err(|e| CliError::Message(format!("write {}: {e}", c_out.display())))?;
    f.write_all(c_src.as_bytes())
        .map_err(|e| CliError::Message(format!("write {}: {e}", c_out.display())))?;

    if let Some(bin) = build {
        build_c_binary(c_out, bin)?;
    }
    Ok(())
}

fn build_c_binary(c_path: &Path, bin: &Path) -> Result<(), CliError> {
    let runtime_dir = runtime_c_dir();
    let runtime_c = runtime_dir.join("runtime.c");
    let status = Command::new("clang")
        .arg("-std=gnu11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_dir)
        .arg(c_path)
        .arg(&runtime_c)
        .arg("-o")
        .arg(bin)
        .status()
        .map_err(|e| CliError::Message(format!("failed to run clang: {e}")))?;

    if !status.success() {
        return Err(CliError::Message(format!(
            "clang failed with status {status}"
        )));
    }
    Ok(())
}

fn load_with_imports(entry: &Path, std_dir: &Path) -> Result<Program, CliError> {
    let mut visited = HashSet::new();
    let mut decls = Vec::new();
    load_recursive(entry, std_dir, &mut visited, &mut decls)?;
    Ok(Program { decls })
}

fn load_recursive(
    path: &Path,
    std_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    out: &mut Vec<Decl>,
) -> Result<(), CliError> {
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
    let print_param = Param {
        mutable: false,
        name: Ident("msg".into()),
        ty: Type::Named(Ident("Str".into())),
    };
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

fn std_dir() -> PathBuf {
    env::var("GAUT_STD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("std"))
}

fn runtime_c_dir() -> PathBuf {
    if let Ok(p) = env::var("GAUT_RUNTIME_C_DIR") {
        return PathBuf::from(p);
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("runtime/c")
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
