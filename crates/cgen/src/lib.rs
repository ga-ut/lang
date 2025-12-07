#![forbid(unsafe_code)]

use frontend::ast::*;
use frontend::parser::Parser;
use std::fmt::{self, Write};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CgenError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("unsupported construct: {0}")]
    Unsupported(String),
    #[error("fmt error: {0}")]
    Fmt(String),
}

pub fn generate_c_from_source(src: &str) -> Result<String, CgenError> {
    let mut parser = Parser::new(src).map_err(|e| CgenError::Parse(e.to_string()))?;
    let program = parser.parse_program().map_err(|e| CgenError::Parse(e.to_string()))?;
    generate_c(&program)
}

pub fn generate_c(program: &Program) -> Result<String, CgenError> {
    let mut out = String::new();
    writeln!(out, "#include <stdint.h>")
        .map_err(|e| CgenError::Fmt(e.to_string()))?;
    writeln!(out, "#include <stdbool.h>\n")
        .map_err(|e| CgenError::Fmt(e.to_string()))?;

    // forward declare type aliases
    for decl in &program.decls {
        if let Decl::Type(t) = decl {
            emit_type_decl(t, &mut out)?;
        }
    }

    // globals
    for decl in &program.decls {
        match decl {
            Decl::Global(b) => emit_global(b, &mut out)?,
            _ => {}
        }
    }

    // functions
    for decl in &program.decls {
        if let Decl::Func(f) = decl {
            emit_function(f, &mut out)?;
        }
    }

    Ok(out)
}

fn emit_type_decl(ty: &TypeDecl, out: &mut String) -> Result<(), CgenError> {
    match &ty.ty {
        Type::Record(fields) => {
            writeln!(out, "typedef struct {{")
                .map_err(|e| CgenError::Fmt(e.to_string()))?;
            for f in fields {
                let cty = map_type(&f.ty)?;
                writeln!(out, "  {} {};", cty, f.name.0)
                    .map_err(|e| CgenError::Fmt(e.to_string()))?;
            }
            writeln!(out, "}} {};", ty.name.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        other => {
            let cty = map_type(other)?;
            writeln!(out, "typedef {} {};", cty, ty.name.0)
                .map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
    }
    writeln!(out).map_err(|e| CgenError::Fmt(e.to_string()))
}

fn emit_global(binding: &Binding, out: &mut String) -> Result<(), CgenError> {
    let cty = map_type(&binding.ty)?;
    write!(out, "{} {} = ", cty, binding.name.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
    emit_expr(&binding.value, out)?;
    writeln!(out, ";\n").map_err(|e| CgenError::Fmt(e.to_string()))
}

fn emit_function(func: &FuncDecl, out: &mut String) -> Result<(), CgenError> {
    let ret_ty = func.ret.clone().unwrap_or(Type::Named(Ident("Unit".into())));
    let ret_cty = if func.name.0 == "main" && matches!(ret_ty, Type::Named(Ident(ref n)) if n == "Unit") {
        "int".to_string()
    } else {
        map_type(&ret_ty)?
    };
    write!(out, "{} {}(", ret_cty, func.name.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
    for (i, p) in func.params.iter().enumerate() {
        if i > 0 {
            write!(out, ", ").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        let cty = map_type(&p.ty)?;
        write!(out, "{} {}", cty, p.name.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    writeln!(out, ") {{").map_err(|e| CgenError::Fmt(e.to_string()))?;

    // simple block body; insert arena stub
    writeln!(out, "  // arena stub").map_err(|e| CgenError::Fmt(e.to_string()))?;
    writeln!(out, "  // arena_reset would be here").map_err(|e| CgenError::Fmt(e.to_string()))?;

    match &func.body {
        Expr::Block(b) => emit_block(b, out, 1)?,
        expr => {
            write!(out, "  return ").map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(expr, out)?;
            writeln!(out, ";").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
    }

    if func.name.0 == "main" && matches!(ret_ty, Type::Named(Ident(ref n)) if n == "Unit") {
        writeln!(out, "  return 0;").map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    writeln!(out, "}}\n").map_err(|e| CgenError::Fmt(e.to_string()))
}

fn emit_block(block: &Block, out: &mut String, indent: usize) -> Result<(), CgenError> {
    let pad = "  ".repeat(indent);
    for stmt in &block.stmts {
        emit_stmt(stmt, out, indent)?;
    }
    if let Some(expr) = &block.tail {
        write!(out, "{}return ", pad).map_err(|e| CgenError::Fmt(e.to_string()))?;
        emit_expr(expr, out)?;
        writeln!(out, ";").map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    Ok(())
}

fn emit_stmt(stmt: &Stmt, out: &mut String, indent: usize) -> Result<(), CgenError> {
    let pad = "  ".repeat(indent);
    match stmt {
        Stmt::Binding(b) => {
            let cty = map_type(&b.ty)?;
            write!(out, "{}{} {} = ", pad, cty, b.name.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&b.value, out)?;
            writeln!(out, ";").map_err(|e| CgenError::Fmt(e.to_string()))?
        }
        Stmt::Assign(a) => {
            write!(out, "{}", pad).map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_path(&a.target, out)?;
            write!(out, " = ").map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&a.value, out)?;
            writeln!(out, ";").map_err(|e| CgenError::Fmt(e.to_string()))?
        }
        Stmt::Expr(e) => {
            write!(out, "{}", pad).map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(e, out)?;
            writeln!(out, ";").map_err(|e| CgenError::Fmt(e.to_string()))?
        }
    }
    Ok(())
}

fn emit_expr(expr: &Expr, out: &mut String) -> Result<(), CgenError> {
    match expr {
        Expr::Literal(l) => match l {
            Literal::Int(i) => write!(out, "{}", i).map_err(|e| CgenError::Fmt(e.to_string()))?,
            Literal::Bool(b) => write!(out, "{}", if *b { "true" } else { "false" }).map_err(|e| CgenError::Fmt(e.to_string()))?,
            Literal::Str(s) => write!(out, "\"{}\"", s).map_err(|e| CgenError::Fmt(e.to_string()))?,
            Literal::Unit => write!(out, "0").map_err(|e| CgenError::Fmt(e.to_string()))?,
        },
        Expr::Path(p) => emit_path(p, out)?,
        Expr::Copy(inner) | Expr::Ref(inner) => emit_expr(inner, out)?,
        Expr::FuncCall(fc) => {
            emit_path(&fc.callee, out)?;
            write!(out, "(").map_err(|e| CgenError::Fmt(e.to_string()))?;
            for (i, arg) in fc.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").map_err(|e| CgenError::Fmt(e.to_string()))?;
                }
                emit_expr(arg, out)?;
            }
            write!(out, ")").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        Expr::If(ife) => {
            write!(out, "(").map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&ife.cond, out)?;
            write!(out, " ? ").map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&ife.then_branch, out)?;
            write!(out, " : ").map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&ife.else_branch, out)?;
            write!(out, ")").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        Expr::Block(b) => {
            write!(out, "({{").map_err(|e| CgenError::Fmt(e.to_string()))?;
            // inline block: emit statements and tail as expression
            for stmt in &b.stmts {
                emit_stmt(stmt, out, 0)?;
            }
            if let Some(tail) = &b.tail {
                emit_expr(tail, out)?;
            } else {
                write!(out, "0").map_err(|e| CgenError::Fmt(e.to_string()))?;
            }
            write!(out, "; }})").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        Expr::RecordLit(r) => {
            // emit as compound literal if possible
            write!(out, "{{ ").map_err(|e| CgenError::Fmt(e.to_string()))?;
            for (i, f) in r.fields.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").map_err(|e| CgenError::Fmt(e.to_string()))?;
                }
                emit_expr(&f.value, out)?;
            }
            write!(out, " }}").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        Expr::Unary(u) => {
            let op = match u.op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
            };
            write!(out, "{}", op).map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&u.expr, out)?;
        }
        Expr::Binary(b) => {
            emit_expr(&b.left, out)?;
            let op = match b.op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Lt => "<",
                BinaryOp::Eq => "==",
                BinaryOp::And => "&&",
                BinaryOp::Or => "||",
            };
            write!(out, " {} ", op).map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&b.right, out)?;
        }
    }
    Ok(())
}

fn emit_path(path: &Path, out: &mut String) -> Result<(), CgenError> {
    for (i, ident) in path.0.iter().enumerate() {
        if i > 0 {
            write!(out, ".").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        write!(out, "{}", ident.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    Ok(())
}

fn map_type(ty: &Type) -> Result<String, CgenError> {
    match ty {
        Type::Named(id) => match id.0.as_str() {
            "i32" => Ok("int32_t".into()),
            "i64" => Ok("int64_t".into()),
            "u8" => Ok("uint8_t".into()),
            "bool" => Ok("bool".into()),
            "Str" => Ok("const char*".into()),
            "Bytes" => Ok("/* bytes */ const char*".into()),
            "Unit" => Ok("void".into()),
            other => Ok(other.to_string()),
        },
        Type::Ref(inner) => map_type(inner),
        Type::Record(_) => Err(CgenError::Unsupported("inline record type".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_program() {
        let src = r#"
        add(a: i32, b: i32) -> i32 = a + b

        main() = {
          x: i32 = 10
          y: i32 = 20
          add(x, y)
        }
        "#;
        let c = generate_c_from_source(src).unwrap();
        assert!(c.contains("int32_t add"));
        assert!(c.contains("int main"));
        assert!(c.contains("return add(x, y);"));
    }
}
