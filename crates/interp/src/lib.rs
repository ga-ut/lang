#![forbid(unsafe_code)]

use frontend::ast::*;
use frontend::parser::Parser;
use indexmap::IndexMap;
use runtime::Arena;
use std::collections::HashMap;
use std::io::{self, Write};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
    Record(IndexMap<String, Value>),
    Unit,
}

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("unknown identifier {0}")]
    UnknownIdent(String),
    #[error("value moved: {0}")]
    Moved(String),
    #[error("not mutable: {0}")]
    NotMutable(String),
    #[error("field not found: {0}")]
    FieldNotFound(String),
    #[error("type error: {0}")]
    Type(String),
}

#[derive(Debug, Clone)]
struct Binding {
    mutable: bool,
    value: Option<Value>, // None indicates moved
}

#[derive(Debug, Clone, Copy)]
enum EvalMode {
    Move,
    Copy,
    Borrow,
}

/// Interpreter with simple block-scoped environment and bump arena per top-level run.
pub struct Interpreter {
    globals: HashMap<String, Binding>,
    funcs: HashMap<String, FuncDecl>,
    arena_cap: usize,
}

impl Interpreter {
    pub fn new(arena_cap: usize) -> Self {
        Self {
            globals: HashMap::new(),
            funcs: HashMap::new(),
            arena_cap,
        }
    }

    pub fn from_source(src: &str) -> Result<Self, RuntimeError> {
        let mut parser = Parser::new(src).map_err(|e| RuntimeError::Type(e.to_string()))?;
        let program = parser
            .parse_program()
            .map_err(|e| RuntimeError::Type(e.to_string()))?;
        let mut interp = Interpreter::new(1024 * 1024);
        interp.load_program(&program)?;
        Ok(interp)
    }

    pub fn load_program(&mut self, program: &Program) -> Result<(), RuntimeError> {
        // collect functions
        for decl in &program.decls {
            if let Decl::Func(f) = decl {
                self.funcs.insert(f.name.0.clone(), f.clone());
            }
        }
        // evaluate globals and lets at top level
        for decl in &program.decls {
            match decl {
                Decl::Global(b) | Decl::Let(b) => {
                    let val = self.eval_expr(
                        &b.value,
                        &mut Env::new_with_arena(self.arena_cap),
                        EvalMode::Move,
                    )?;
                    self.globals.insert(
                        b.name.0.clone(),
                        Binding {
                            mutable: b.mutable,
                            value: Some(val),
                        },
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Evaluate `main()` and return its result value.
    pub fn run_main(&mut self) -> Result<Value, RuntimeError> {
        let Some(main_fn) = self.funcs.get("main").cloned() else {
            return Err(RuntimeError::UnknownIdent("main".into()));
        };
        let mut env = Env::new_with_arena(self.arena_cap);
        env.init_globals(&self.globals);
        self.call_function(&main_fn, vec![], &mut env)
    }

    fn call_function(
        &mut self,
        func: &FuncDecl,
        args: Vec<Value>,
        env: &mut Env,
    ) -> Result<Value, RuntimeError> {
        if func.params.len() != args.len() {
            return Err(RuntimeError::Type("arity mismatch".into()));
        }
        env.push_scope();
        for (param, arg) in func.params.iter().zip(args.into_iter()) {
            env.insert_binding(
                param.name.0.clone(),
                Binding {
                    mutable: param.mutable,
                    value: Some(arg),
                },
            );
        }

        let result = match &func.body {
            Expr::Block(b) => self.eval_block(b, env)?,
            other => self.eval_expr(other, env, EvalMode::Move)?,
        };
        env.pop_scope();
        Ok(result)
    }

    fn eval_block(&mut self, block: &Block, env: &mut Env) -> Result<Value, RuntimeError> {
        env.push_scope();
        for stmt in &block.stmts {
            self.eval_stmt(stmt, env)?;
        }
        let result = if let Some(expr) = &block.tail {
            self.eval_expr(expr, env, EvalMode::Move)?
        } else {
            Value::Unit
        };
        env.pop_scope();
        Ok(result)
    }

    fn eval_stmt(&mut self, stmt: &Stmt, env: &mut Env) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Binding(b) => {
                let val = self.eval_expr(&b.value, env, EvalMode::Move)?;
                env.insert_binding(
                    b.name.0.clone(),
                    Binding {
                        mutable: b.mutable,
                        value: Some(val),
                    },
                );
                Ok(())
            }
            Stmt::Assign(a) => {
                let val = self.eval_expr(&a.value, env, EvalMode::Move)?;
                env.assign_path(&a.target, val)
            }
            Stmt::Expr(e) => {
                let _ = self.eval_expr(e, env, EvalMode::Move)?;
                Ok(())
            }
        }
    }

    fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &mut Env,
        mode: EvalMode,
    ) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(l) => Ok(match l {
                Literal::Int(v) => Value::Int(*v),
                Literal::Bool(b) => Value::Bool(*b),
                Literal::Str(s) => Value::Str(s.clone()),
                Literal::Unit => Value::Unit,
            }),
            Expr::Path(p) => env.resolve_path(p, mode),
            Expr::Copy(inner) => {
                let v = self.eval_expr(inner, env, EvalMode::Copy)?;
                Ok(v)
            }
            Expr::Ref(inner) => {
                // For now, treat ref as borrow-copy (no mutation through ref in 1st version).
                let v = self.eval_expr(inner, env, EvalMode::Borrow)?;
                Ok(v)
            }
            Expr::FuncCall(fc) => {
                let func_name = path_to_string(&fc.callee);
                if let Some(func) = self.funcs.get(&func_name).cloned() {
                    let mut args = Vec::new();
                    for a in &fc.args {
                        args.push(self.eval_expr(a, env, EvalMode::Move)?);
                    }
                    self.call_function(&func, args, env)
                } else if let Some(res) = eval_builtin(&func_name, &fc.args, self, env)? {
                    Ok(res)
                } else {
                    Err(RuntimeError::UnknownIdent(func_name))
                }
            }
            Expr::If(ife) => {
                let cond = self.eval_expr(&ife.cond, env, EvalMode::Move)?;
                match cond {
                    Value::Bool(true) => self.eval_expr(&ife.then_branch, env, EvalMode::Move),
                    Value::Bool(false) => self.eval_expr(&ife.else_branch, env, EvalMode::Move),
                    _ => Err(RuntimeError::Type("if condition must be bool".into())),
                }
            }
            Expr::Block(b) => self.eval_block(b, env),
            Expr::RecordLit(r) => {
                let mut map = IndexMap::new();
                for f in &r.fields {
                    let v = self.eval_expr(&f.value, env, EvalMode::Move)?;
                    map.insert(f.name.0.clone(), v);
                }
                Ok(Value::Record(map))
            }
            Expr::Unary(u) => {
                let v = self.eval_expr(&u.expr, env, EvalMode::Move)?;
                match (u.op.clone(), v) {
                    (UnaryOp::Neg, Value::Int(i)) => Ok(Value::Int(-i)),
                    (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    _ => Err(RuntimeError::Type("invalid unary operand".into())),
                }
            }
            Expr::Binary(b) => {
                let l = self.eval_expr(&b.left, env, EvalMode::Move)?;
                let r = self.eval_expr(&b.right, env, EvalMode::Move)?;
                self.eval_binary(&l, &r, b.op.clone())
            }
        }
    }

    fn eval_binary(&self, l: &Value, r: &Value, op: BinaryOp) -> Result<Value, RuntimeError> {
        match op {
            BinaryOp::Add => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                _ => Err(RuntimeError::Type("invalid operands for +".into())),
            },
            BinaryOp::Sub => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                _ => Err(RuntimeError::Type("invalid operands for -".into())),
            },
            BinaryOp::Mul => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                _ => Err(RuntimeError::Type("invalid operands for *".into())),
            },
            BinaryOp::Div => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
                _ => Err(RuntimeError::Type("invalid operands for /".into())),
            },
            BinaryOp::Lt => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                _ => Err(RuntimeError::Type("invalid operands for <".into())),
            },
            BinaryOp::Eq => Ok(Value::Bool(l == r)),
            BinaryOp::And => match (l, r) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
                _ => Err(RuntimeError::Type("invalid operands for &&".into())),
            },
            BinaryOp::Or => match (l, r) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
                _ => Err(RuntimeError::Type("invalid operands for ||".into())),
            },
        }
    }
}

fn eval_builtin(
    name: &str,
    args: &[Expr],
    interp: &mut Interpreter,
    env: &mut Env,
) -> Result<Option<Value>, RuntimeError> {
    match name {
        "print" | "println" => {
            if args.len() != 1 {
                return Err(RuntimeError::Type(
                    "print/println expects one argument".into(),
                ));
            }
            let val = interp.eval_expr(&args[0], env, EvalMode::Move)?;
            let s = match val {
                Value::Str(ref s) => s.clone(),
                other => format!("{other:?}"),
            };
            if name == "print" {
                print!("{}", s);
                io::stdout().flush().ok();
            } else {
                println!("{}", s);
            }
            Ok(Some(Value::Str(s)))
        }
        _ => Ok(None),
    }
}

#[derive(Debug)]
struct Env {
    scopes: Vec<HashMap<String, Binding>>, // innermost at end
    arena: Arena,
}

impl Env {
    fn new_with_arena(cap: usize) -> Self {
        Self {
            scopes: Vec::new(),
            arena: Arena::with_capacity(cap),
        }
    }

    fn init_globals(&mut self, globals: &HashMap<String, Binding>) {
        self.push_scope();
        if let Some(scope) = self.scopes.last_mut() {
            for (k, v) in globals.iter() {
                scope.insert(k.clone(), v.clone());
            }
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        self.arena.reset();
    }

    fn insert_binding(&mut self, name: String, binding: Binding) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, binding);
        } else {
            self.scopes.push(HashMap::from([(name, binding)]));
        }
    }

    fn resolve_path(&mut self, path: &Path, mode: EvalMode) -> Result<Value, RuntimeError> {
        let (head, rest) = path
            .0
            .split_first()
            .ok_or_else(|| RuntimeError::UnknownIdent("".into()))?;
        // find binding from innermost to outer
        let mut idx = self.scopes.len();
        let mut binding_idx = None;
        while idx > 0 {
            idx -= 1;
            if self.scopes[idx].contains_key(&head.0) {
                binding_idx = Some(idx);
                break;
            }
        }
        let Some(scope_idx) = binding_idx else {
            return Err(RuntimeError::UnknownIdent(head.0.clone()));
        };
        let scope = self.scopes.get_mut(scope_idx).unwrap();
        let binding = scope.get_mut(&head.0).unwrap();

        match mode {
            EvalMode::Move => {
                let mut val = binding
                    .value
                    .take()
                    .ok_or_else(|| RuntimeError::Moved(head.0.clone()))?;
                for field in rest {
                    val = extract_field(val, &field.0)?;
                }
                Ok(val)
            }
            EvalMode::Copy | EvalMode::Borrow => {
                let val = binding
                    .value
                    .as_ref()
                    .ok_or_else(|| RuntimeError::Moved(head.0.clone()))?;
                let mut out = val.clone();
                for field in rest {
                    out = extract_field(out, &field.0)?;
                }
                Ok(out)
            }
        }
    }

    fn assign_path(&mut self, path: &Path, value: Value) -> Result<(), RuntimeError> {
        let (head, rest) = path
            .0
            .split_first()
            .ok_or_else(|| RuntimeError::UnknownIdent("".into()))?;
        let mut idx = self.scopes.len();
        let mut binding_idx = None;
        while idx > 0 {
            idx -= 1;
            if self.scopes[idx].contains_key(&head.0) {
                binding_idx = Some(idx);
                break;
            }
        }
        let Some(scope_idx) = binding_idx else {
            return Err(RuntimeError::UnknownIdent(head.0.clone()));
        };
        let scope = self.scopes.get_mut(scope_idx).unwrap();
        let binding = scope.get_mut(&head.0).unwrap();
        if !binding.mutable {
            return Err(RuntimeError::NotMutable(head.0.clone()));
        }
        let Some(slot) = binding.value.as_mut() else {
            return Err(RuntimeError::Moved(head.0.clone()));
        };

        if rest.is_empty() {
            *slot = value;
            return Ok(());
        }

        set_field(slot, rest, value)
    }
}

fn extract_field(val: Value, field: &str) -> Result<Value, RuntimeError> {
    match val {
        Value::Record(mut m) => m
            .shift_remove(field)
            .ok_or_else(|| RuntimeError::FieldNotFound(field.into())),
        _ => Err(RuntimeError::Type("field access on non-record".into())),
    }
}

fn set_field(target: &mut Value, path: &[Ident], value: Value) -> Result<(), RuntimeError> {
    if path.is_empty() {
        *target = value;
        return Ok(());
    }
    match target {
        Value::Record(ref mut m) => {
            let key = path[0].0.clone();
            if path.len() == 1 {
                if let Some(slot) = m.get_mut(&key) {
                    *slot = value;
                    Ok(())
                } else {
                    Err(RuntimeError::FieldNotFound(key))
                }
            } else {
                let next = m
                    .get_mut(&key)
                    .ok_or_else(|| RuntimeError::FieldNotFound(key.clone()))?;
                set_field(next, &path[1..], value)
            }
        }
        _ => Err(RuntimeError::Type(
            "assignment into non-record field".into(),
        )),
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

    fn run(src: &str) -> Value {
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        let mut interp = Interpreter::new(1024 * 1024);
        interp.load_program(&program).unwrap();
        interp.run_main().unwrap()
    }

    #[test]
    fn calc_example() {
        let src = r#"
        add(a: i32, b: i32) -> i32 = a + b

        main() = {
          x: i32 = 10
          y: i32 = 20
          add(x, y)
        }
        "#;
        let v = run(src);
        assert_eq!(v, Value::Int(30));
    }

    #[test]
    fn record_ref_example() {
        let src = r#"
        type Point = { x: i32, y: i32 }

        length_x(p: &Point) -> i32 = p.x

        main() = {
          origin: Point = { x: 0, y: 0 }
          length_x(&origin)
        }
        "#;
        let v = run(src);
        assert_eq!(v, Value::Int(0));
    }

    #[test]
    fn if_block_and_move() {
        let src = r#"
        main() = {
          x: i32 = 1
          y: i32 = if x < 0 then 10 else 5
          y
        }
        "#;
        let v = run(src);
        assert_eq!(v, Value::Int(5));
    }
}
