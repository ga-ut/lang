#![forbid(unsafe_code)]

use crate::ast::*;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TypeError {
    #[error("unknown identifier {0}")]
    UnknownIdent(String),
    #[error("unknown type {0}")]
    UnknownType(String),
    #[error("unknown function {0}")]
    UnknownFunc(String),
    #[error("cannot infer return type for function {0} yet")]
    UnknownFuncReturn(String),
    #[error("type mismatch: expected {expected:?}, found {found:?}")]
    TypeMismatch { expected: Type, found: Type },
    #[error("function arity mismatch: expected {expected}, found {found}")]
    ArityMismatch { expected: usize, found: usize },
    #[error("value moved: {0}")]
    Moved(String),
    #[error("assignment to immutable binding: {0}")]
    NotMutable(String),
    #[error("value escapes its defining block")]
    Escape,
    #[error("main must not take parameters")]
    MainHasParams,
}

#[derive(Debug, Clone)]
struct BindingInfo {
    ty: Type,
    mutable: bool,
    moved: bool,
    origin_depth: usize,
}

#[derive(Debug, Clone)]
struct Scope {
    vars: HashMap<String, BindingInfo>,
}

#[derive(Debug, Clone)]
struct FuncSig {
    params: Vec<Param>,
    ret: Option<Type>,
}

pub struct TypeChecker {
    types: HashMap<String, Type>,
    funcs: HashMap<String, FuncSig>,
    scopes: Vec<Scope>,
    builtins: HashSet<String>,
}

#[derive(Debug, Clone)]
struct TyInfo {
    ty: Type,
    origin_depth: usize,
    escapable: bool, // whether this value may legally escape its origin block (when refs are absent)
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut types = HashMap::new();
        for name in ["i32", "i64", "u8", "bool", "Str", "Bytes", "Unit"] {
            types.insert(name.to_string(), Type::Named(Ident(name.to_string())));
        }
        types.insert(
            "ReadFileResult".into(),
            Type::Record(vec![
                FieldType {
                    name: Ident("ok".into()),
                    ty: Type::Named(Ident("bool".into())),
                },
                FieldType {
                    name: Ident("data".into()),
                    ty: Type::Named(Ident("Str".into())),
                },
            ]),
        );
        let builtins = types.keys().cloned().collect();

        let mut funcs = HashMap::new();
        funcs.insert(
            "print".into(),
            FuncSig {
                params: vec![Param {
                    mutable: false,
                    name: Ident("msg".into()),
                    ty: Type::Named(Ident("Str".into())),
                }],
                ret: Some(Type::Named(Ident("Str".into()))),
            },
        );
        funcs.insert(
            "println".into(),
            FuncSig {
                params: vec![Param {
                    mutable: false,
                    name: Ident("msg".into()),
                    ty: Type::Named(Ident("Str".into())),
                }],
                ret: Some(Type::Named(Ident("Str".into()))),
            },
        );
        funcs.insert(
            "read_file".into(),
            FuncSig {
                params: vec![Param {
                    mutable: false,
                    name: Ident("path".into()),
                    ty: Type::Named(Ident("Str".into())),
                }],
                ret: Some(Type::Named(Ident("Str".into()))),
            },
        );
        funcs.insert(
            "write_file".into(),
            FuncSig {
                params: vec![
                    Param {
                        mutable: false,
                        name: Ident("path".into()),
                        ty: Type::Named(Ident("Str".into())),
                    },
                    Param {
                        mutable: false,
                        name: Ident("data".into()),
                        ty: Type::Named(Ident("Str".into())),
                    },
                ],
                ret: Some(Type::Named(Ident("Unit".into()))),
            },
        );
        funcs.insert(
            "args".into(),
            FuncSig {
                params: Vec::new(),
                ret: Some(Type::Named(Ident("Bytes".into()))),
            },
        );
        funcs.insert(
            "bytes_to_str".into(),
            FuncSig {
                params: vec![Param {
                    mutable: false,
                    name: Ident("buf".into()),
                    ty: Type::Named(Ident("Bytes".into())),
                }],
                ret: Some(Type::Named(Ident("Str".into()))),
            },
        );
        funcs.insert(
            "try_read_file".into(),
            FuncSig {
                params: vec![Param {
                    mutable: false,
                    name: Ident("path".into()),
                    ty: Type::Named(Ident("Str".into())),
                }],
                ret: Some(Type::Named(Ident("ReadFileResult".into()))),
            },
        );
        funcs.insert(
            "try_write_file".into(),
            FuncSig {
                params: vec![
                    Param {
                        mutable: false,
                        name: Ident("path".into()),
                        ty: Type::Named(Ident("Str".into())),
                    },
                    Param {
                        mutable: false,
                        name: Ident("data".into()),
                        ty: Type::Named(Ident("Str".into())),
                    },
                ],
                ret: Some(Type::Named(Ident("bool".into()))),
            },
        );
        funcs.insert(
            "str_len".into(),
            FuncSig {
                params: vec![Param {
                    mutable: false,
                    name: Ident("s".into()),
                    ty: Type::Named(Ident("Str".into())),
                }],
                ret: Some(Type::Named(Ident("i32".into()))),
            },
        );
        funcs.insert(
            "str_byte_at".into(),
            FuncSig {
                params: vec![
                    Param {
                        mutable: false,
                        name: Ident("s".into()),
                        ty: Type::Named(Ident("Str".into())),
                    },
                    Param {
                        mutable: false,
                        name: Ident("i".into()),
                        ty: Type::Named(Ident("i32".into())),
                    },
                ],
                ret: Some(Type::Named(Ident("i32".into()))),
            },
        );
        funcs.insert(
            "str_slice".into(),
            FuncSig {
                params: vec![
                    Param {
                        mutable: false,
                        name: Ident("s".into()),
                        ty: Type::Named(Ident("Str".into())),
                    },
                    Param {
                        mutable: false,
                        name: Ident("start".into()),
                        ty: Type::Named(Ident("i32".into())),
                    },
                    Param {
                        mutable: false,
                        name: Ident("len".into()),
                        ty: Type::Named(Ident("i32".into())),
                    },
                ],
                ret: Some(Type::Named(Ident("Str".into()))),
            },
        );

        Self {
            types,
            funcs,
            scopes: Vec::new(),
            builtins,
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), TypeError> {
        // pass 1: collect type aliases and function signatures
        for decl in &program.decls {
            match decl {
                Decl::Type(t) => {
                    self.types.insert(t.name.0.clone(), t.ty.clone());
                }
                Decl::Func(f) => {
                    let ret = f.ret.clone();
                    self.funcs.insert(
                        f.name.0.clone(),
                        FuncSig {
                            params: f.params.clone(),
                            ret,
                        },
                    );
                }
                _ => {}
            }
        }

        // global scope
        self.push_scope();

        let mut funcs_to_check: Vec<FuncDecl> = Vec::new();
        for decl in &program.decls {
            match decl {
                Decl::Import(_) => {}
                Decl::Type(_) => {}
                Decl::Func(f) => funcs_to_check.push(f.clone()),
                Decl::Global(b) | Decl::Let(b) => {
                    self.check_binding(b, 0)?;
                }
            }
        }

        let mut pending = funcs_to_check;
        while !pending.is_empty() {
            let mut deferred: Vec<FuncDecl> = Vec::new();
            let mut progressed = false;
            for func in pending {
                match self.check_func(&func) {
                    Ok(()) => progressed = true,
                    Err(TypeError::UnknownFuncReturn(_)) => deferred.push(func),
                    Err(err) => return Err(err),
                }
            }
            if !progressed {
                let unresolved = deferred
                    .first()
                    .map(|f| f.name.0.clone())
                    .unwrap_or_else(|| "<unknown>".to_string());
                return Err(TypeError::UnknownFuncReturn(unresolved));
            }
            pending = deferred;
        }

        Ok(())
    }

    fn check_func(&mut self, func: &FuncDecl) -> Result<(), TypeError> {
        if func.name.0 == "main" && !func.params.is_empty() {
            return Err(TypeError::MainHasParams);
        }
        let sig = self
            .funcs
            .get(&func.name.0)
            .cloned()
            .ok_or_else(|| TypeError::UnknownFunc(func.name.0.clone()))?;

        self.push_scope();
        let result = (|| {
            let depth = self.current_depth();
            for p in &sig.params {
                let ty = self.resolve_type(&p.ty)?;
                self.insert_var(p.name.0.clone(), ty, p.mutable, depth);
            }
            let body_info = match &func.body {
                Expr::Block(b) => self.check_block(b, true)?,
                other => self.check_expr(other, ValueMode::Move)?,
            };
            self.ensure_not_escape(&body_info, depth)?;

            let inferred_ret = if let Some(ref annotated) = sig.ret {
                self.ensure_type(annotated, &body_info.ty)?;
                annotated.clone()
            } else {
                body_info.ty.clone()
            };
            // update function signature with inferred return for downstream calls
            if let Some(entry) = self.funcs.get_mut(&func.name.0) {
                entry.ret = Some(inferred_ret);
            }
            Ok(())
        })();
        self.pop_scope();
        result
    }

    fn check_binding(&mut self, binding: &Binding, depth: usize) -> Result<(), TypeError> {
        let ty_ann = self.resolve_type(&binding.ty)?;
        let value = self.check_expr(&binding.value, ValueMode::Move)?;
        self.ensure_not_escape(&value, depth)?;
        self.ensure_type(&ty_ann, &value.ty)?;
        self.insert_var(binding.name.0.clone(), ty_ann, binding.mutable, depth);
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        match stmt {
            Stmt::Binding(b) => {
                let depth = self.current_depth();
                self.check_binding(b, depth)
            }
            Stmt::Assign(a) => self.check_assign(a),
            Stmt::Expr(e) => {
                self.check_expr(e, ValueMode::Move)?;
                Ok(())
            }
        }
    }

    fn check_assign(&mut self, assign: &Assign) -> Result<(), TypeError> {
        let (binding_depth, info) = self.lookup_binding(&assign.target)?;
        if !info.mutable {
            return Err(TypeError::NotMutable(path_to_string(&assign.target)));
        }
        let value = self.check_expr(&assign.value, ValueMode::Move)?;
        self.ensure_not_escape(&value, binding_depth)?;
        self.ensure_type(&info.ty, &value.ty)?;
        // mark the binding as refreshed (not moved)
        self.set_moved(&assign.target, false)?;
        Ok(())
    }

    fn check_block(
        &mut self,
        block: &Block,
        allow_escape_values: bool,
    ) -> Result<TyInfo, TypeError> {
        self.push_scope();
        let depth = self.current_depth();
        for stmt in &block.stmts {
            self.check_stmt(stmt)?;
        }
        let tail_ty = if let Some(expr) = &block.tail {
            let info = self.check_expr(expr, ValueMode::Move)?;
            if info.origin_depth > depth {
                if !allow_escape_values || type_contains_ref(&info.ty) || !info.escapable {
                    return Err(TypeError::Escape);
                }
            } else {
                self.ensure_not_escape(&info, depth)?;
            }
            if allow_escape_values {
                // normalize origin to this depth; escapable only if it has no refs
                let ty_clone = info.ty.clone();
                TyInfo {
                    ty: info.ty,
                    origin_depth: depth,
                    escapable: !type_contains_ref(&ty_clone),
                }
            } else {
                // value produced in an inner block expression should not be allowed to escape further
                TyInfo {
                    ty: info.ty,
                    origin_depth: info.origin_depth,
                    escapable: false,
                }
            }
        } else {
            TyInfo {
                ty: Type::Named(Ident("Unit".into())),
                origin_depth: depth,
                escapable: true,
            }
        };
        self.pop_scope();
        Ok(tail_ty)
    }

    fn check_expr(&mut self, expr: &Expr, mode: ValueMode) -> Result<TyInfo, TypeError> {
        match expr {
            Expr::Literal(l) => Ok(TyInfo {
                ty: literal_type(l),
                origin_depth: self.current_depth(),
                escapable: true,
            }),
            Expr::Path(p) => self.eval_path(p, mode),
            Expr::Copy(inner) => {
                let info = self.check_expr(inner, ValueMode::Copy)?;
                Ok(info)
            }
            Expr::Ref(inner) => {
                let info = self.check_expr(inner, ValueMode::Borrow)?;
                Ok(TyInfo {
                    ty: Type::Ref(Box::new(info.ty)),
                    origin_depth: info.origin_depth,
                    escapable: info.escapable,
                })
            }
            Expr::FuncCall(fc) => self.eval_call(fc),
            Expr::If(ifexpr) => {
                let cond = self.check_expr(&ifexpr.cond, ValueMode::Move)?;
                self.ensure_type(&Type::Named(Ident("bool".into())), &cond.ty)?;
                let t = self.check_expr(&ifexpr.then_branch, ValueMode::Move)?;
                let e = self.check_expr(&ifexpr.else_branch, ValueMode::Move)?;
                self.ensure_type(&t.ty, &e.ty)?;
                Ok(TyInfo {
                    ty: t.ty,
                    origin_depth: std::cmp::max(t.origin_depth, e.origin_depth),
                    escapable: t.escapable && e.escapable,
                })
            }
            Expr::Block(b) => self.check_block(b, false),
            Expr::RecordLit(r) => {
                let mut fields = Vec::new();
                let mut max_depth = self.current_depth();
                let mut escapable = true;
                for f in &r.fields {
                    let val = self.check_expr(&f.value, ValueMode::Move)?;
                    max_depth = max_depth.max(val.origin_depth);
                    escapable = escapable && val.escapable;
                    fields.push(FieldType {
                        name: f.name.clone(),
                        ty: val.ty,
                    });
                }
                Ok(TyInfo {
                    ty: Type::Record(fields),
                    origin_depth: max_depth,
                    escapable,
                })
            }
            Expr::Unary(u) => {
                let val = self.check_expr(&u.expr, ValueMode::Move)?;
                match u.op {
                    UnaryOp::Neg => self.ensure_type(&Type::Named(Ident("i32".into())), &val.ty)?,
                    UnaryOp::Not => {
                        self.ensure_type(&Type::Named(Ident("bool".into())), &val.ty)?
                    }
                }
                Ok(val)
            }
            Expr::Binary(b) => {
                let l = self.check_expr(&b.left, ValueMode::Move)?;
                let r = self.check_expr(&b.right, ValueMode::Move)?;
                match b.op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                        // allow i32 math, and Str + Str as concatenation (other combos are errors)
                        let escapable = l.escapable && r.escapable;
                        if self.type_eq(&l.ty, &Type::Named(Ident("i32".into())))?
                            && self.type_eq(&r.ty, &Type::Named(Ident("i32".into())))?
                        {
                            Ok(TyInfo {
                                ty: Type::Named(Ident("i32".into())),
                                origin_depth: std::cmp::max(l.origin_depth, r.origin_depth),
                                escapable,
                            })
                        } else if self.type_eq(&l.ty, &Type::Named(Ident("Str".into())))?
                            && self.type_eq(&r.ty, &Type::Named(Ident("Str".into())))?
                        {
                            Ok(TyInfo {
                                ty: Type::Named(Ident("Str".into())),
                                origin_depth: std::cmp::max(l.origin_depth, r.origin_depth),
                                escapable,
                            })
                        } else {
                            Err(TypeError::TypeMismatch {
                                expected: l.ty.clone(),
                                found: r.ty.clone(),
                            })
                        }
                    }
                    BinaryOp::Lt | BinaryOp::Eq => {
                        self.ensure_type(&l.ty, &r.ty)?;
                        Ok(TyInfo {
                            ty: Type::Named(Ident("bool".into())),
                            origin_depth: std::cmp::max(l.origin_depth, r.origin_depth),
                            escapable: l.escapable && r.escapable,
                        })
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        self.ensure_type(&Type::Named(Ident("bool".into())), &l.ty)?;
                        self.ensure_type(&Type::Named(Ident("bool".into())), &r.ty)?;
                        Ok(TyInfo {
                            ty: Type::Named(Ident("bool".into())),
                            origin_depth: std::cmp::max(l.origin_depth, r.origin_depth),
                            escapable: l.escapable && r.escapable,
                        })
                    }
                }
            }
        }
    }

    fn eval_path(&mut self, path: &Path, mode: ValueMode) -> Result<TyInfo, TypeError> {
        let (_depth, info) = self.lookup_binding(path)?;
        match mode {
            ValueMode::Move => {
                if info.moved {
                    return Err(TypeError::Moved(path_to_string(path)));
                }
                self.set_moved(path, true)?;
            }
            ValueMode::Copy | ValueMode::Borrow => {
                if info.moved {
                    return Err(TypeError::Moved(path_to_string(path)));
                }
            }
        }
        Ok(TyInfo {
            ty: info.ty.clone(),
            origin_depth: info.origin_depth,
            escapable: false,
        })
    }

    fn eval_call(&mut self, call: &FuncCall) -> Result<TyInfo, TypeError> {
        if call.callee.0.len() != 1 {
            return Err(TypeError::UnknownFunc(path_to_string(&call.callee)));
        }
        let name = call.callee.0[0].0.clone();
        let sig = self
            .funcs
            .get(&name)
            .ok_or_else(|| TypeError::UnknownFunc(name.clone()))?
            .clone();
        if sig.params.len() != call.args.len() {
            return Err(TypeError::ArityMismatch {
                expected: sig.params.len(),
                found: call.args.len(),
            });
        }
        for (arg_expr, param) in call.args.iter().zip(sig.params.iter()) {
            let arg = self.check_expr(arg_expr, ValueMode::Move)?;
            let pty = self.resolve_type(&param.ty)?;
            self.ensure_type(&pty, &arg.ty)?;
        }
        let ret_ty = sig
            .ret
            .clone()
            .ok_or_else(|| TypeError::UnknownFuncReturn(name.clone()))?;
        Ok(TyInfo {
            ty: ret_ty.clone(),
            origin_depth: self.current_depth(),
            escapable: !type_contains_ref(&ret_ty),
        })
    }

    fn ensure_type(&self, expected: &Type, found: &Type) -> Result<(), TypeError> {
        if self.type_eq(expected, found)? {
            Ok(())
        } else {
            Err(TypeError::TypeMismatch {
                expected: expected.clone(),
                found: found.clone(),
            })
        }
    }

    fn ensure_not_escape(&self, info: &TyInfo, target_depth: usize) -> Result<(), TypeError> {
        if info.origin_depth > target_depth {
            if !info.escapable || type_contains_ref(&info.ty) {
                return Err(TypeError::Escape);
            }
        }
        Ok(())
    }

    fn type_eq(&self, a: &Type, b: &Type) -> Result<bool, TypeError> {
        let ra = self.resolve_type(a)?;
        let rb = self.resolve_type(b)?;
        Ok(match (ra, rb) {
            (Type::Named(x), Type::Named(y)) => x == y,
            (Type::Ref(ax), Type::Ref(bx)) => self.type_eq(&ax, &bx)?,
            (Type::Record(af), Type::Record(bf)) => {
                if af.len() != bf.len() {
                    false
                } else {
                    af.iter().zip(bf.iter()).all(|(a, b)| {
                        a.name == b.name && self.type_eq(&a.ty, &b.ty).unwrap_or(false)
                    })
                }
            }
            _ => false,
        })
    }

    fn resolve_type(&self, ty: &Type) -> Result<Type, TypeError> {
        match ty {
            Type::Named(id) => {
                if let Some(t) = self.types.get(&id.0) {
                    if self.builtins.contains(&id.0) {
                        Ok(t.clone())
                    } else {
                        // expand aliases
                        Ok(self.resolve_type(t)?)
                    }
                } else {
                    Err(TypeError::UnknownType(id.0.clone()))
                }
            }
            Type::Ref(inner) => Ok(Type::Ref(Box::new(self.resolve_type(inner)?))),
            Type::Record(fields) => {
                let mut out = Vec::new();
                for f in fields {
                    out.push(FieldType {
                        name: f.name.clone(),
                        ty: self.resolve_type(&f.ty)?,
                    });
                }
                Ok(Type::Record(out))
            }
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            vars: HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_depth(&self) -> usize {
        self.scopes.len().saturating_sub(1)
    }

    fn insert_var(&mut self, name: String, ty: Type, mutable: bool, origin_depth: usize) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.vars.insert(
                name,
                BindingInfo {
                    ty,
                    mutable,
                    moved: false,
                    origin_depth,
                },
            );
        }
    }

    fn lookup_binding(&self, path: &Path) -> Result<(usize, BindingInfo), TypeError> {
        let (head, rest) = path
            .0
            .split_first()
            .ok_or_else(|| TypeError::UnknownIdent("".into()))?;
        for (depth_rev, scope) in self.scopes.iter().rev().enumerate() {
            if let Some(info) = scope.vars.get(&head.0) {
                let depth = self.scopes.len().saturating_sub(1) - depth_rev;
                let mut ty = info.ty.clone();
                for field in rest {
                    // unwrap references transparently during field access
                    loop {
                        match ty {
                            Type::Ref(inner) => {
                                ty = *inner.clone();
                                continue;
                            }
                            _ => break,
                        }
                    }

                    match ty {
                        Type::Record(ref fields) => {
                            if let Some(ft) = fields.iter().find(|f| f.name == *field) {
                                ty = ft.ty.clone();
                            } else {
                                return Err(TypeError::UnknownIdent(field.0.clone()));
                            }
                        }
                        _ => return Err(TypeError::UnknownIdent(field.0.clone())),
                    }
                }
                return Ok((
                    depth,
                    BindingInfo {
                        ty,
                        mutable: info.mutable,
                        moved: info.moved,
                        origin_depth: info.origin_depth,
                    },
                ));
            }
        }
        Err(TypeError::UnknownIdent(head.0.clone()))
    }

    fn set_moved(&mut self, path: &Path, moved: bool) -> Result<(), TypeError> {
        let (head, rest) = path
            .0
            .split_first()
            .ok_or_else(|| TypeError::UnknownIdent("".into()))?;
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.vars.get_mut(&head.0) {
                if !rest.is_empty() {
                    // moving through record moves whole binding
                    info.moved = moved;
                } else {
                    info.moved = moved;
                }
                return Ok(());
            }
        }
        Err(TypeError::UnknownIdent(head.0.clone()))
    }
}

#[derive(Debug, Clone, Copy)]
enum ValueMode {
    Move,
    Copy,
    Borrow,
}

fn literal_type(lit: &Literal) -> Type {
    match lit {
        Literal::Int(_) => Type::Named(Ident("i32".into())),
        Literal::Bool(_) => Type::Named(Ident("bool".into())),
        Literal::Str(_) => Type::Named(Ident("Str".into())),
        Literal::Unit => Type::Named(Ident("Unit".into())),
    }
}

fn type_contains_ref(ty: &Type) -> bool {
    match ty {
        Type::Ref(_) => true,
        Type::Record(fields) => fields.iter().any(|f| type_contains_ref(&f.ty)),
        _ => false,
    }
}

fn path_to_string(path: &Path) -> String {
    path.0
        .iter()
        .map(|i| i.0.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn check_ok(src: &str) {
        let mut parser = Parser::new(src).expect("parser init");
        let program = parser.parse_program().expect("parse program");
        let mut tc = TypeChecker::new();
        tc.check_program(&program).expect("typecheck ok");
    }

    fn check_err(src: &str) -> TypeError {
        let mut parser = Parser::new(src).expect("parser init");
        let program = parser.parse_program().expect("parse program");
        let mut tc = TypeChecker::new();
        tc.check_program(&program).expect_err("expected type error")
    }

    #[test]
    fn success_hello() {
        let src = r#"
        global greeting: Str = "hello"

        print(msg: Str) = msg

        main() = {
          msg: Str = greeting + " world"
          print(msg)
        }
        "#;
        check_ok(src);
    }

    #[test]
    fn success_calc() {
        let src = r#"
        add(a: i32, b: i32) -> i32 = a + b

        main() = {
          x: i32 = 1
          y: i32 = 2
          sum: i32 = add(x, y)
          copy sum
        }
        "#;
        check_ok(src);
    }

    #[test]
    fn success_forward_call_with_inferred_return() {
        let src = r#"
        main() = {
          out: i32 = id(7)
          copy out
        }

        id(x: i32) = x
        "#;
        check_ok(src);
    }

    #[test]
    fn success_infer_function_return_type() {
        let src = r#"
        id(x: i32) = x

        main() = {
          out: i32 = id(7)
          copy out
        }
        "#;
        check_ok(src);
    }

    #[test]
    fn success_record_ref() {
        let src = r#"
        type Point = { x: i32, y: i32 }

        length_x(p: &Point) -> i32 = p.x

        main() = {
          origin: Point = { x: 0, y: 0 }
          px: i32 = length_x(&origin)
          copy px
        }
        "#;
        check_ok(src);
    }

    #[test]
    fn fail_use_after_move() {
        let src = r#"
        main() = {
          x: i32 = 1
          y: i32 = x
          x
        }
        "#;
        let err = check_err(src);
        assert!(matches!(err, TypeError::Moved(_)));
    }

    #[test]
    fn fail_escape_block() {
        let src = r#"
        main() = {
          y: i32 = { x: i32 = 1 x }
          y
        }
        "#;
        let err = check_err(src);
        assert!(matches!(err, TypeError::Escape));
    }

    #[test]
    fn fail_type_mismatch() {
        let src = r#"
        add(a: i32, b: i32) -> i32 = a + b

        main() = {
          msg: Str = "hi"
          add(msg, msg)
        }
        "#;
        let err = check_err(src);
        assert!(matches!(err, TypeError::TypeMismatch { .. }));
    }
}
