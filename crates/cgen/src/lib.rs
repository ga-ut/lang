#![forbid(unsafe_code)]

use frontend::ast::*;
use frontend::parser::Parser;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CgenError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("unsupported construct: {0}")]
    Unsupported(String),
    #[error("fmt error: {0}")]
    Fmt(String),
    #[error("unknown identifier in codegen: {0}")]
    UnknownIdent(String),
}

#[derive(Debug, Clone)]
struct FuncSig {
    ret: Option<Type>,
}

#[derive(Debug, Default, Clone)]
struct Counters {
    tmp: usize,
    scope: usize,
}

#[derive(Debug, Clone)]
struct TypeCtx {
    types: HashMap<String, Type>,
    funcs: HashMap<String, FuncSig>,
    scopes: Vec<HashMap<String, Type>>, // innermost last
}

impl TypeCtx {
    fn new(program: &Program) -> Self {
        let mut types = HashMap::new();
        for name in ["i32", "i64", "u8", "bool", "Str", "Bytes", "Unit"] {
            types.insert(name.to_string(), Type::Named(Ident(name.to_string())));
        }

        let mut funcs = HashMap::new();
        for decl in &program.decls {
            if let Decl::Func(f) = decl {
                funcs.insert(f.name.0.clone(), FuncSig { ret: f.ret.clone() });
            }
            if let Decl::Type(t) = decl {
                types.insert(t.name.0.clone(), t.ty.clone());
            }
        }
        // Builtins
        funcs.entry("print".into()).or_insert(FuncSig {
            ret: Some(Type::Named(Ident("Str".into()))),
        });
        funcs.entry("println".into()).or_insert(FuncSig {
            ret: Some(Type::Named(Ident("Str".into()))),
        });
        funcs.entry("read_file".into()).or_insert(FuncSig {
            ret: Some(Type::Named(Ident("Str".into()))),
        });
        funcs.entry("write_file".into()).or_insert(FuncSig {
            ret: Some(Type::Named(Ident("Unit".into()))),
        });
        funcs.entry("args".into()).or_insert(FuncSig {
            ret: Some(Type::Named(Ident("Bytes".into()))),
        });

        let mut ctx = Self {
            types,
            funcs,
            scopes: Vec::new(),
        };
        ctx.push_scope();
        for decl in &program.decls {
            if let Decl::Global(b) | Decl::Let(b) = decl {
                ctx.insert_var(b.name.0.clone(), b.ty.clone());
            }
        }
        ctx
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_var(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn resolve_alias(&self, ty: &Type) -> Type {
        let mut current = ty.clone();
        let mut seen = HashSet::new();
        loop {
            match current {
                Type::Named(ref id) => {
                    if !seen.insert(id.0.clone()) {
                        return current;
                    }
                    if let Some(t) = self.types.get(&id.0) {
                        current = t.clone();
                        continue;
                    }
                    return current;
                }
                Type::Ref(inner) => return Type::Ref(Box::new(self.resolve_alias(&inner))),
                Type::Record(_) => return current,
            }
        }
    }

    fn type_of_ident(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }

    fn type_of_path(&self, path: &Path) -> Option<Type> {
        let (head, rest) = path.0.split_first()?;
        let mut ty = self.type_of_ident(&head.0)?;
        for field in rest {
            ty = self.field_type(&ty, &field.0)?;
        }
        Some(ty)
    }

    fn field_type(&self, ty: &Type, field: &str) -> Option<Type> {
        match self.resolve_alias(ty) {
            Type::Record(fields) => fields
                .iter()
                .find(|f| f.name.0 == field)
                .map(|f| f.ty.clone()),
            Type::Ref(inner) => self.field_type(&inner, field),
            _ => None,
        }
    }

    fn infer_expr_type(&self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Literal(Literal::Int(_)) => Some(Type::Named(Ident("i32".into()))),
            Expr::Literal(Literal::Bool(_)) => Some(Type::Named(Ident("bool".into()))),
            Expr::Literal(Literal::Str(_)) => Some(Type::Named(Ident("Str".into()))),
            Expr::Literal(Literal::Unit) => Some(Type::Named(Ident("Unit".into()))),
            Expr::Path(p) => self.type_of_path(p),
            Expr::Copy(inner) => self.infer_expr_type(inner),
            Expr::Ref(inner) => self.infer_expr_type(inner).map(|t| Type::Ref(Box::new(t))),
            Expr::FuncCall(fc) => {
                let name = path_to_string(&fc.callee);
                self.funcs.get(&name).and_then(|f| {
                    f.ret
                        .clone()
                        .or_else(|| Some(Type::Named(Ident("Unit".into()))))
                })
            }
            Expr::If(ife) => {
                let then_ty = self.infer_expr_type(&ife.then_branch)?;
                let else_ty = self.infer_expr_type(&ife.else_branch)?;
                if then_ty == else_ty {
                    Some(then_ty)
                } else {
                    Some(Type::Named(Ident("Unit".into())))
                }
            }
            Expr::Block(b) => self.infer_block_type(b),
            Expr::RecordLit(r) => {
                let mut fields = Vec::new();
                for f in &r.fields {
                    let ty = self
                        .infer_expr_type(&f.value)
                        .unwrap_or(Type::Named(Ident("Unit".into())));
                    fields.push(FieldType {
                        name: f.name.clone(),
                        ty,
                    });
                }
                Some(Type::Record(fields))
            }
            Expr::Unary(u) => match u.op {
                UnaryOp::Neg => Some(Type::Named(Ident("i32".into()))),
                UnaryOp::Not => Some(Type::Named(Ident("bool".into()))),
            },
            Expr::Binary(b) => {
                let lhs = self.infer_expr_type(&b.left)?;
                let rhs = self.infer_expr_type(&b.right)?;
                match b.op {
                    BinaryOp::Lt | BinaryOp::Eq | BinaryOp::And | BinaryOp::Or => {
                        Some(Type::Named(Ident("bool".into())))
                    }
                    BinaryOp::Add => {
                        if self.is_str(&lhs) || self.is_str(&rhs) {
                            Some(Type::Named(Ident("Str".into())))
                        } else {
                            Some(lhs)
                        }
                    }
                    _ => Some(lhs),
                }
            }
        }
    }

    fn infer_block_type(&self, block: &Block) -> Option<Type> {
        let mut clone = self.clone();
        clone.push_scope();
        for stmt in &block.stmts {
            clone.infer_stmt(stmt);
        }
        let tail_ty = block
            .tail
            .as_ref()
            .map(|e| clone.infer_expr_type(e))
            .flatten()
            .unwrap_or(Type::Named(Ident("Unit".into())));
        Some(tail_ty)
    }

    fn infer_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Binding(b) => {
                self.insert_var(b.name.0.clone(), b.ty.clone());
            }
            _ => {}
        }
    }

    fn is_str(&self, ty: &Type) -> bool {
        matches!(self.resolve_alias(ty), Type::Named(Ident(ref n)) if n == "Str")
    }

    fn is_bytes(&self, ty: &Type) -> bool {
        matches!(self.resolve_alias(ty), Type::Named(Ident(ref n)) if n == "Bytes")
    }

    fn is_unit(&self, ty: &Type) -> bool {
        matches!(self.resolve_alias(ty), Type::Named(Ident(ref n)) if n == "Unit")
    }
}

pub fn generate_c_from_source(src: &str) -> Result<String, CgenError> {
    let mut parser = Parser::new(src).map_err(|e| CgenError::Parse(e.to_string()))?;
    let program = parser
        .parse_program()
        .map_err(|e| CgenError::Parse(e.to_string()))?;
    generate_c(&program)
}

pub fn generate_c(program: &Program) -> Result<String, CgenError> {
    let mut ctx = TypeCtx::new(program);
    let mut out = String::new();
    writeln!(out, "#include <stdint.h>").map_err(|e| CgenError::Fmt(e.to_string()))?;
    writeln!(out, "#include <stdbool.h>").map_err(|e| CgenError::Fmt(e.to_string()))?;
    writeln!(out, "#include <stddef.h>").map_err(|e| CgenError::Fmt(e.to_string()))?;
    writeln!(out, "#include \"runtime.h\"\n").map_err(|e| CgenError::Fmt(e.to_string()))?;

    let mut func_names = HashSet::new();
    for decl in &program.decls {
        if let Decl::Func(f) = decl {
            func_names.insert(f.name.0.clone());
        }
    }
    emit_builtin_shims(&mut out, &func_names)?;

    // forward declare type aliases
    for decl in &program.decls {
        if let Decl::Type(t) = decl {
            emit_type_decl(t, &mut out, &mut ctx)?;
        }
    }

    // globals (let/global)
    for decl in &program.decls {
        if let Decl::Global(b) | Decl::Let(b) = decl {
            emit_global(b, &mut out, &mut ctx)?;
        }
    }

    // functions
    for decl in &program.decls {
        if let Decl::Func(f) = decl {
            emit_function(f, &mut out, &mut ctx)?;
        }
    }

    Ok(out)
}

fn emit_builtin_shims(out: &mut String, func_names: &HashSet<String>) -> Result<(), CgenError> {
    if !func_names.contains("print") {
        writeln!(
            out,
            "char* print(char* msg) {{ gaut_print(msg); return msg; }}"
        )
        .map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    if !func_names.contains("println") {
        writeln!(
            out,
            "char* println(char* msg) {{ gaut_println(msg); return msg; }}"
        )
        .map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    if !func_names.contains("read_file") {
        writeln!(
            out,
            "char* read_file(char* path) {{ return gaut_read_file(path); }}"
        )
        .map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    if !func_names.contains("write_file") {
        writeln!(
            out,
            "void write_file(char* path, char* data) {{ gaut_write_file(path, data); }}"
        )
        .map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    if !func_names.contains("args") {
        writeln!(out, "gaut_bytes args() {{ return gaut_args(); }}")
            .map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    writeln!(out).map_err(|e| CgenError::Fmt(e.to_string()))
}

fn emit_type_decl(ty: &TypeDecl, out: &mut String, ctx: &mut TypeCtx) -> Result<(), CgenError> {
    match ctx.resolve_alias(&ty.ty) {
        Type::Record(fields) => {
            writeln!(out, "typedef struct {{").map_err(|e| CgenError::Fmt(e.to_string()))?;
            for f in fields {
                let cty = map_type(&f.ty, ctx)?;
                writeln!(out, "  {} {};", cty, f.name.0)
                    .map_err(|e| CgenError::Fmt(e.to_string()))?;
            }
            writeln!(out, "}} {};", ty.name.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        other => {
            let cty = map_type(&other, ctx)?;
            writeln!(out, "typedef {} {};", cty, ty.name.0)
                .map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
    }
    writeln!(out).map_err(|e| CgenError::Fmt(e.to_string()))
}

fn emit_global(binding: &Binding, out: &mut String, ctx: &mut TypeCtx) -> Result<(), CgenError> {
    let cty = map_value_type(&binding.ty, ctx)?;
    write!(out, "{} {} = ", cty, binding.name.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
    let mut ctrs = Counters::default();
    emit_expr(&binding.value, out, ctx, None, &mut ctrs)?;
    writeln!(out, ";\n").map_err(|e| CgenError::Fmt(e.to_string()))
}

fn emit_function(func: &FuncDecl, out: &mut String, ctx: &mut TypeCtx) -> Result<(), CgenError> {
    if func.name.0 == "print" || func.name.0 == "println" {
        emit_builtin_print(func, out, ctx)?;
        return Ok(());
    }
    if func.name.0 == "read_file" || func.name.0 == "write_file" || func.name.0 == "args" {
        emit_builtin_io(func, out, ctx)?;
        return Ok(());
    }

    let mut infer_ctx = ctx.clone();
    infer_ctx.push_scope();
    for p in &func.params {
        infer_ctx.insert_var(p.name.0.clone(), p.ty.clone());
    }
    let inferred_ret = infer_ctx
        .infer_expr_type(&func.body)
        .unwrap_or(Type::Named(Ident("Unit".into())));
    let ret_ty = func.ret.clone().unwrap_or(inferred_ret);
    let returns_unit = ctx.is_unit(&ret_ty);
    let ret_cty = if func.name.0 == "main" {
        "int".to_string()
    } else if returns_unit {
        map_type(&ret_ty, ctx)?
    } else {
        map_type(&ret_ty, ctx)?
    };

    write!(out, "{} {}(", ret_cty, func.name.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
    for (i, p) in func.params.iter().enumerate() {
        if i > 0 {
            write!(out, ", ").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        let cty = map_value_type(&p.ty, ctx)?;
        write!(out, "{} {}", cty, p.name.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    writeln!(out, ") {{").map_err(|e| CgenError::Fmt(e.to_string()))?;

    ctx.push_scope();
    for p in &func.params {
        ctx.insert_var(p.name.0.clone(), p.ty.clone());
    }

    writeln!(out, "  uint8_t __arena_buf[GAUT_DEFAULT_ARENA_CAP];")
        .map_err(|e| CgenError::Fmt(e.to_string()))?;
    writeln!(
        out,
        "  gaut_arena __arena = gaut_arena_from_buffer(__arena_buf, GAUT_DEFAULT_ARENA_CAP);"
    )
    .map_err(|e| CgenError::Fmt(e.to_string()))?;
    writeln!(out).map_err(|e| CgenError::Fmt(e.to_string()))?;

    let mut counters = Counters::default();
    let body_block = match &func.body {
        Expr::Block(b) => b.clone(),
        other => Block {
            stmts: Vec::new(),
            tail: Some(Box::new(other.clone())),
        },
    };
    emit_block(
        &body_block,
        out,
        ctx,
        1,
        &ret_ty,
        Some("__arena"),
        func.name.0 == "main",
        &mut counters,
    )?;

    ctx.pop_scope();
    writeln!(out, "}}\n").map_err(|e| CgenError::Fmt(e.to_string()))
}

fn emit_builtin_print(func: &FuncDecl, out: &mut String, ctx: &TypeCtx) -> Result<(), CgenError> {
    let name = &func.name.0;
    let ret_cty = map_type(&Type::Named(Ident("Str".into())), ctx)?;
    let msg_cty = map_type(&Type::Named(Ident("Str".into())), ctx)?;
    writeln!(out, "{} {}({} msg) {{", ret_cty, name, msg_cty)
        .map_err(|e| CgenError::Fmt(e.to_string()))?;
    let call = if name == "print" {
        "gaut_print"
    } else {
        "gaut_println"
    };
    writeln!(out, "  {}(msg);", call).map_err(|e| CgenError::Fmt(e.to_string()))?;
    writeln!(out, "  return msg;").map_err(|e| CgenError::Fmt(e.to_string()))?;
    writeln!(out, "}}\n").map_err(|e| CgenError::Fmt(e.to_string()))
}

fn emit_builtin_io(func: &FuncDecl, out: &mut String, ctx: &TypeCtx) -> Result<(), CgenError> {
    match func.name.0.as_str() {
        "read_file" => {
            let ret_cty = map_type(&Type::Named(Ident("Str".into())), ctx)?;
            let path_cty = map_type(&Type::Named(Ident("Str".into())), ctx)?;
            writeln!(out, "{} read_file({} path) {{", ret_cty, path_cty)
                .map_err(|e| CgenError::Fmt(e.to_string()))?;
            writeln!(out, "  return gaut_read_file(path);")
                .map_err(|e| CgenError::Fmt(e.to_string()))?;
            writeln!(out, "}}\n").map_err(|e| CgenError::Fmt(e.to_string()))
        }
        "write_file" => {
            writeln!(out, "void write_file(char* path, char* data) {{")
                .map_err(|e| CgenError::Fmt(e.to_string()))?;
            writeln!(out, "  gaut_write_file(path, data);")
                .map_err(|e| CgenError::Fmt(e.to_string()))?;
            writeln!(out, "}}\n").map_err(|e| CgenError::Fmt(e.to_string()))
        }
        "args" => {
            let ret_cty = map_type(&Type::Named(Ident("Bytes".into())), ctx)?;
            writeln!(out, "{} args() {{", ret_cty).map_err(|e| CgenError::Fmt(e.to_string()))?;
            writeln!(out, "  return gaut_args();").map_err(|e| CgenError::Fmt(e.to_string()))?;
            writeln!(out, "}}\n").map_err(|e| CgenError::Fmt(e.to_string()))
        }
        _ => Ok(()),
    }
}

fn emit_block(
    block: &Block,
    out: &mut String,
    ctx: &mut TypeCtx,
    indent: usize,
    ret_ty: &Type,
    arena: Option<&str>,
    is_main: bool,
    ctrs: &mut Counters,
) -> Result<(), CgenError> {
    let pad = "  ".repeat(indent);
    ctx.push_scope();
    let scope_name = if let Some(a) = arena {
        let name = format!("__scope{}", ctrs.scope);
        ctrs.scope += 1;
        writeln!(
            out,
            "{}gaut_scope {} = gaut_scope_enter(&{});",
            pad, name, a
        )
        .map_err(|e| CgenError::Fmt(e.to_string()))?;
        Some(name)
    } else {
        None
    };
    for stmt in &block.stmts {
        emit_stmt(stmt, out, ctx, indent, arena, ctrs)?;
    }
    if let Some(expr) = &block.tail {
        let ret_expr_arena = if ctx.is_str(ret_ty) || ctx.is_bytes(ret_ty) {
            None
        } else {
            arena
        };
        if ctx.is_unit(ret_ty) {
            write!(out, "{}", pad).map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(expr, out, ctx, ret_expr_arena, ctrs)?;
            writeln!(out, ";").map_err(|e| CgenError::Fmt(e.to_string()))?;
            if let (Some(a), Some(s)) = (arena, &scope_name) {
                writeln!(out, "{}gaut_scope_leave(&{}, {});", pad, a, s)
                    .map_err(|e| CgenError::Fmt(e.to_string()))?;
            }
            if is_main {
                writeln!(out, "{}return 0;", pad).map_err(|e| CgenError::Fmt(e.to_string()))?;
            }
        } else {
            let cty = map_value_type(ret_ty, ctx)?;
            let tmp = format!("__ret{}", ctrs.tmp);
            ctrs.tmp += 1;
            write!(out, "{}{} {} = ", pad, cty, tmp).map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(expr, out, ctx, ret_expr_arena, ctrs)?;
            writeln!(out, ";").map_err(|e| CgenError::Fmt(e.to_string()))?;
            if let (Some(a), Some(s)) = (arena, &scope_name) {
                writeln!(out, "{}gaut_scope_leave(&{}, {});", pad, a, s)
                    .map_err(|e| CgenError::Fmt(e.to_string()))?;
            }
            writeln!(out, "{}return {};", pad, tmp).map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
    } else {
        if !ctx.is_unit(ret_ty) {
            return Err(CgenError::Unsupported("missing return expression".into()));
        }
        if let (Some(a), Some(s)) = (arena, &scope_name) {
            writeln!(out, "{}gaut_scope_leave(&{}, {});", pad, a, s)
                .map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        if is_main {
            writeln!(out, "{}return 0;", pad).map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
    }
    ctx.pop_scope();
    Ok(())
}

fn emit_stmt(
    stmt: &Stmt,
    out: &mut String,
    ctx: &mut TypeCtx,
    indent: usize,
    arena: Option<&str>,
    ctrs: &mut Counters,
) -> Result<(), CgenError> {
    let pad = "  ".repeat(indent);
    match stmt {
        Stmt::Binding(b) => {
            let cty = map_value_type(&b.ty, ctx)?;
            write!(out, "{}{} {} = ", pad, cty, b.name.0)
                .map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&b.value, out, ctx, arena, ctrs)?;
            writeln!(out, ";").map_err(|e| CgenError::Fmt(e.to_string()))?;
            ctx.insert_var(b.name.0.clone(), b.ty.clone());
        }
        Stmt::Assign(a) => {
            write!(out, "{}", pad).map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_path(&a.target, out, Some(&*ctx))?;
            write!(out, " = ").map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&a.value, out, ctx, arena, ctrs)?;
            writeln!(out, ";").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        Stmt::Expr(e) => {
            write!(out, "{}", pad).map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(e, out, ctx, arena, ctrs)?;
            writeln!(out, ";").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
    }
    Ok(())
}

fn emit_expr(
    expr: &Expr,
    out: &mut String,
    ctx: &mut TypeCtx,
    arena: Option<&str>,
    ctrs: &mut Counters,
) -> Result<Type, CgenError> {
    match expr {
        Expr::Literal(l) => match l {
            Literal::Int(i) => write!(out, "{}", i).map_err(|e| CgenError::Fmt(e.to_string()))?,
            Literal::Bool(b) => write!(out, "{}", if *b { "true" } else { "false" })
                .map_err(|e| CgenError::Fmt(e.to_string()))?,
            Literal::Str(s) => {
                write!(out, "\"{}\"", s).map_err(|e| CgenError::Fmt(e.to_string()))?
            }
            Literal::Unit => write!(out, "0").map_err(|e| CgenError::Fmt(e.to_string()))?,
        },
        Expr::Path(p) => {
            emit_path(p, out, Some(&*ctx))?;
        }
        Expr::Copy(inner) => {
            return emit_expr(inner, out, ctx, arena, ctrs);
        }
        Expr::Ref(inner) => {
            write!(out, "&").map_err(|e| CgenError::Fmt(e.to_string()))?;
            return emit_expr(inner, out, ctx, arena, ctrs);
        }
        Expr::FuncCall(fc) => {
            emit_path(&fc.callee, out, None)?;
            write!(out, "(").map_err(|e| CgenError::Fmt(e.to_string()))?;
            for (i, arg) in fc.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").map_err(|e| CgenError::Fmt(e.to_string()))?;
                }
                emit_expr(arg, out, ctx, arena, ctrs)?;
            }
            write!(out, ")").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        Expr::If(ife) => {
            write!(out, "(").map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&ife.cond, out, ctx, arena, ctrs)?;
            write!(out, " ? ").map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&ife.then_branch, out, ctx, arena, ctrs)?;
            write!(out, " : ").map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&ife.else_branch, out, ctx, arena, ctrs)?;
            write!(out, ")").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        Expr::Block(b) => {
            let ty = emit_block_expr(b, out, ctx, arena, ctrs)?;
            return Ok(ty);
        }
        Expr::RecordLit(r) => {
            let ty = ctx
                .infer_expr_type(expr)
                .unwrap_or(Type::Record(Vec::new()));
            let cty = find_record_alias(ctx, &ty).unwrap_or(map_value_type(&ty, ctx)?);
            write!(out, "({}){{ ", cty).map_err(|e| CgenError::Fmt(e.to_string()))?;
            for (i, f) in r.fields.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").map_err(|e| CgenError::Fmt(e.to_string()))?;
                }
                write!(out, ".{} = ", f.name.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
                emit_expr(&f.value, out, ctx, arena, ctrs)?;
            }
            write!(out, " }}").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        Expr::Unary(u) => {
            let op = match u.op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
            };
            write!(out, "{}", op).map_err(|e| CgenError::Fmt(e.to_string()))?;
            emit_expr(&u.expr, out, ctx, arena, ctrs)?;
        }
        Expr::Binary(b) => {
            let ty = ctx.infer_expr_type(expr);
            if matches!(b.op, BinaryOp::Add) && ty.as_ref().is_some_and(|t| ctx.is_str(t)) {
                let fn_name = if arena.is_some() {
                    "gaut_str_concat_arena"
                } else {
                    "gaut_str_concat_heap"
                };
                if let Some(a) = arena {
                    write!(out, "{}(&{}, ", fn_name, a)
                        .map_err(|e| CgenError::Fmt(e.to_string()))?;
                } else {
                    write!(out, "{}(", fn_name).map_err(|e| CgenError::Fmt(e.to_string()))?;
                }
                emit_expr(&b.left, out, ctx, arena, ctrs)?;
                write!(out, ", ").map_err(|e| CgenError::Fmt(e.to_string()))?;
                emit_expr(&b.right, out, ctx, arena, ctrs)?;
                write!(out, ")").map_err(|e| CgenError::Fmt(e.to_string()))?;
            } else if matches!(b.op, BinaryOp::Add) && ty.as_ref().is_some_and(|t| ctx.is_bytes(t))
            {
                let fn_name = if arena.is_some() {
                    "gaut_bytes_concat_arena"
                } else {
                    "gaut_bytes_concat_heap"
                };
                if let Some(a) = arena {
                    write!(out, "{}(&{}, ", fn_name, a)
                        .map_err(|e| CgenError::Fmt(e.to_string()))?;
                } else {
                    write!(out, "{}(", fn_name).map_err(|e| CgenError::Fmt(e.to_string()))?;
                }
                emit_expr(&b.left, out, ctx, arena, ctrs)?;
                write!(out, ", ").map_err(|e| CgenError::Fmt(e.to_string()))?;
                emit_expr(&b.right, out, ctx, arena, ctrs)?;
                write!(out, ")").map_err(|e| CgenError::Fmt(e.to_string()))?;
            } else {
                emit_expr(&b.left, out, ctx, arena, ctrs)?;
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
                emit_expr(&b.right, out, ctx, arena, ctrs)?;
            }
        }
    }

    Ok(ctx
        .infer_expr_type(expr)
        .unwrap_or(Type::Named(Ident("Unit".into()))))
}

fn emit_block_expr(
    block: &Block,
    out: &mut String,
    ctx: &mut TypeCtx,
    arena: Option<&str>,
    ctrs: &mut Counters,
) -> Result<Type, CgenError> {
    let ty = ctx
        .infer_block_type(block)
        .unwrap_or(Type::Named(Ident("Unit".into())));
    let tmp = format!("__tmp{}", ctrs.tmp);
    ctrs.tmp += 1;
    write!(out, "({{ ").map_err(|e| CgenError::Fmt(e.to_string()))?;
    ctx.push_scope();
    let scope_name = if let Some(a) = arena {
        let name = format!("__scope{}", ctrs.scope);
        ctrs.scope += 1;
        write!(out, "gaut_scope {} = gaut_scope_enter(&{}); ", name, a)
            .map_err(|e| CgenError::Fmt(e.to_string()))?;
        Some(name)
    } else {
        None
    };
    for stmt in &block.stmts {
        emit_stmt(stmt, out, ctx, 0, arena, ctrs)?;
    }
    let cty = map_value_type(&ty, ctx)?;
    if let Some(tail) = &block.tail {
        write!(out, "{} {} = ", cty, tmp).map_err(|e| CgenError::Fmt(e.to_string()))?;
        emit_expr(tail, out, ctx, arena, ctrs)?;
        write!(out, "; ").map_err(|e| CgenError::Fmt(e.to_string()))?;
    } else {
        write!(out, "{} {} = 0; ", cty, tmp).map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    if let (Some(a), Some(s)) = (arena, &scope_name) {
        write!(out, "gaut_scope_leave(&{}, {}); ", a, s)
            .map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    ctx.pop_scope();
    write!(out, "{}; }})", tmp).map_err(|e| CgenError::Fmt(e.to_string()))?;
    Ok(ty)
}

fn emit_path(path: &Path, out: &mut String, ctx: Option<&TypeCtx>) -> Result<(), CgenError> {
    if let (Some(tc), Some((head, rest))) = (ctx, path.0.split_first()) {
        let mut current = tc.type_of_ident(&head.0);
        write!(out, "{}", head.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
        for field in rest {
            if let Some(ref ty) = current {
                let resolved = tc.resolve_alias(ty);
                match resolved {
                    Type::Ref(inner) => {
                        write!(out, "->{}", field.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
                        current = tc.field_type(&inner, &field.0);
                    }
                    _ => {
                        write!(out, ".{}", field.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
                        current = tc.field_type(ty, &field.0);
                    }
                }
            } else {
                write!(out, ".{}", field.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
                current = None;
            }
        }
        return Ok(());
    }

    for (i, ident) in path.0.iter().enumerate() {
        if i > 0 {
            write!(out, ".").map_err(|e| CgenError::Fmt(e.to_string()))?;
        }
        write!(out, "{}", ident.0).map_err(|e| CgenError::Fmt(e.to_string()))?;
    }
    Ok(())
}

fn is_builtin_name(name: &str) -> bool {
    matches!(
        name,
        "i32" | "i64" | "u8" | "bool" | "Str" | "Bytes" | "Unit"
    )
}

fn find_record_alias(ctx: &TypeCtx, ty: &Type) -> Option<String> {
    let Type::Record(fields) = ctx.resolve_alias(ty) else {
        return None;
    };
    for (name, aliased) in &ctx.types {
        if is_builtin_name(name) {
            continue;
        }
        if let Type::Record(alias_fields) = ctx.resolve_alias(aliased) {
            if fields.len() != alias_fields.len() {
                continue;
            }
            let mut same = true;
            for (a, b) in fields.iter().zip(alias_fields.iter()) {
                if a.name != b.name || ctx.resolve_alias(&a.ty) != ctx.resolve_alias(&b.ty) {
                    same = false;
                    break;
                }
            }
            if same {
                return Some(name.clone());
            }
        }
    }
    None
}

fn map_value_type(ty: &Type, ctx: &TypeCtx) -> Result<String, CgenError> {
    match ty {
        Type::Named(id) => {
            let resolved = ctx.resolve_alias(ty);
            if matches!(resolved, Type::Named(Ident(ref n)) if n == "Unit") {
                return Ok("int".into());
            }
            match id.0.as_str() {
                "i32" => Ok("int32_t".into()),
                "i64" => Ok("int64_t".into()),
                "u8" => Ok("uint8_t".into()),
                "bool" => Ok("bool".into()),
                "Str" => Ok("char*".into()),
                "Bytes" => Ok("gaut_bytes".into()),
                other => Ok(other.to_string()),
            }
        }
        Type::Ref(inner) => Ok(format!("{}*", map_value_type(inner, ctx)?)),
        Type::Record(fields) => {
            let mut tmp = String::new();
            writeln!(tmp, "struct {{").map_err(|e| CgenError::Fmt(e.to_string()))?;
            for f in fields {
                let cty = map_value_type(&f.ty, ctx)?;
                writeln!(tmp, "  {} {};", cty, f.name.0)
                    .map_err(|e| CgenError::Fmt(e.to_string()))?;
            }
            write!(tmp, "}}").map_err(|e| CgenError::Fmt(e.to_string()))?;
            Ok(tmp)
        }
    }
}

fn map_type(ty: &Type, ctx: &TypeCtx) -> Result<String, CgenError> {
    match ty {
        Type::Named(id) => match id.0.as_str() {
            "i32" => Ok("int32_t".into()),
            "i64" => Ok("int64_t".into()),
            "u8" => Ok("uint8_t".into()),
            "bool" => Ok("bool".into()),
            "Str" => Ok("char*".into()),
            "Bytes" => Ok("gaut_bytes".into()),
            "Unit" => Ok("void".into()),
            other => Ok(other.to_string()),
        },
        Type::Ref(inner) => Ok(format!("{}*", map_type(&inner, ctx)?)),
        Type::Record(fields) => {
            let mut tmp = String::new();
            writeln!(tmp, "struct {{").map_err(|e| CgenError::Fmt(e.to_string()))?;
            for f in fields {
                let cty = map_type(&f.ty, ctx)?;
                writeln!(tmp, "  {} {};", cty, f.name.0)
                    .map_err(|e| CgenError::Fmt(e.to_string()))?;
            }
            write!(tmp, "}}").map_err(|e| CgenError::Fmt(e.to_string()))?;
            Ok(tmp)
        }
    }
}

fn path_to_string(path: &Path) -> String {
    path.0
        .iter()
        .map(|i| i.0.clone())
        .collect::<Vec<_>>()
        .join(".")
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
        assert!(c.contains("main()"));
        assert!(c.contains("gaut_arena __arena"));
        assert!(c.contains("add(x, y)"));
    }

    #[test]
    fn string_concat_uses_runtime() {
        let src = r#"
        main() = {
          msg: Str = "hello" + " world"
          msg
        }
        "#;
        let c = generate_c_from_source(src).unwrap();
        assert!(c.contains("gaut_str_concat"));
    }

    #[test]
    fn record_ref_uses_arrow() {
        let src = r#"
        type Point = { x: i32, y: i32 }

        length_x(p: &Point) -> i32 = p.x

        main() = {
          origin: Point = { x: 0, y: 0 }
          px: i32 = length_x(&origin)
          px
        }
        "#;
        let c = generate_c_from_source(src).unwrap();
        assert!(c.contains("p->x"));
        assert!(c.contains("Point origin"));
    }

    #[test]
    fn read_file_calls_runtime() {
        let src = r#"
        main() = {
          content: Str = read_file("foo.txt")
          content
        }
        "#;
        let c = generate_c_from_source(src).unwrap();
        assert!(c.contains("gaut_read_file"));
    }
}
