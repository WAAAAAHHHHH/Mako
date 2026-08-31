use crate::ast::{BinaryOp, Expr, Program, Stmt};
use std::collections::HashMap;

// ── Control flow signals ─────────────────────────────────────────────────────

#[derive(Debug)]
enum Signal {
    Stop,
    Skip,
    Give(Value),
    Throw(Value),
}

#[derive(Debug)]
enum ExecResult {
    Ok,
    Err(String),
    Signal(Signal),
}

// ── Values ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Tuple(Vec<Value>),
    Object {
        type_name: std::string::String,
        fields: HashMap<std::string::String, Value>,
    },
    TypeDef {
        name: std::string::String,
        methods: HashMap<std::string::String, (Vec<std::string::String>, Stmt)>,
    },
    Function {
        params: Vec<std::string::String>,
        body: Box<Stmt>,
    },
    Nothing,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::List(l) => {
                let items: Vec<std::string::String> = l.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Map(m) => {
                let mut pairs: Vec<std::string::String> =
                    m.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                pairs.sort();
                write!(f, "{{{}}}", pairs.join(", "))
            }
            Value::Tuple(t) => {
                let items: Vec<std::string::String> = t.iter().map(|v| v.to_string()).collect();
                write!(f, "({})", items.join(", "))
            }
            Value::Object { type_name, fields } => {
                let mut pairs: Vec<std::string::String> =
                    fields.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                pairs.sort();
                write!(f, "<{} {{{}}}>", type_name, pairs.join(", "))
            }
            Value::TypeDef { name, .. } => write!(f, "<type {}>", name),
            Value::Function { params, .. } => write!(f, "<fn({})>", params.join(", ")),
            Value::Nothing => write!(f, "nothing"),
        }
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Number(n) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Nothing => false,
        Value::List(l) => !l.is_empty(),
        Value::Map(m) => !m.is_empty(),
        Value::Tuple(t) => !t.is_empty(),
        _ => true,
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Nothing, Value::Nothing) => true,
        _ => false,
    }
}

// ── Runtime ──────────────────────────────────────────────────────────────────

pub struct Runtime {
    environments: Vec<HashMap<std::string::String, (Value, bool)>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            environments: vec![HashMap::new()],
        }
    }

    pub fn execute(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            match self.exec_stmt(stmt) {
                ExecResult::Ok => {}
                ExecResult::Err(e) => return Err(e),
                ExecResult::Signal(Signal::Give(_)) => {
                    return Err("Error: 'give' used outside of a function".to_string());
                }
                ExecResult::Signal(Signal::Stop) => {
                    return Err("Error: 'stop' used outside of a loop".to_string());
                }
                ExecResult::Signal(Signal::Skip) => {
                    return Err("Error: 'skip' used outside of a loop".to_string());
                }
                ExecResult::Signal(Signal::Throw(v)) => {
                    return Err(format!("Uncaught error: {}", v));
                }
            }
        }
        Ok(())
    }

    // ── Environment helpers ──────────────────────────────────────────────────

    fn lookup(&self, name: &str) -> Option<Value> {
        for env in self.environments.iter().rev() {
            if let Some((val, _)) = env.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    fn declare(&mut self, name: std::string::String, val: Value, is_const: bool) -> Result<(), String> {
        let env = self.environments.last_mut().unwrap();
        if env.contains_key(&name) {
            return Err(format!("Error: `{}` is already declared in this scope", name));
        }
        env.insert(name, (val, is_const));
        Ok(())
    }

    fn assign(&mut self, name: &str, val: Value) -> Result<(), String> {
        for env in self.environments.iter_mut().rev() {
            if let Some((_, is_const)) = env.get(name) {
                if *is_const {
                    return Err(format!("Error: cannot reassign constant `{}`", name));
                }
                env.insert(name.to_string(), (val, false));
                return Ok(());
            }
        }
        Err(format!(
            "Error: `{}` has not been declared\nHint: use `!{} = ...` to declare it",
            name, name
        ))
    }

    // ── Statement execution ──────────────────────────────────────────────────

    fn exec_stmt(&mut self, stmt: &Stmt) -> ExecResult {
        match stmt {
            Stmt::VarDecl { name, init } => match self.evaluate_expression(init) {
                Ok(val) => match self.declare(name.clone(), val, false) {
                    Ok(()) => ExecResult::Ok,
                    Err(e) => ExecResult::Err(e),
                },
                Err(e) => ExecResult::Err(e),
            },

            Stmt::ConstDecl { name, init } => match self.evaluate_expression(init) {
                Ok(val) => match self.declare(name.clone(), val, true) {
                    Ok(()) => ExecResult::Ok,
                    Err(e) => ExecResult::Err(e),
                },
                Err(e) => ExecResult::Err(e),
            },

            Stmt::Assign { name, expr } => match self.evaluate_expression(expr) {
                Ok(val) => match self.assign(name, val) {
                    Ok(()) => ExecResult::Ok,
                    Err(e) => ExecResult::Err(e),
                },
                Err(e) => ExecResult::Err(e),
            },

            Stmt::IndexAssign { name, index, value } => {
                let idx_val = match self.evaluate_expression(index) {
                    Ok(v) => v,
                    Err(e) => return ExecResult::Err(e),
                };
                let new_val = match self.evaluate_expression(value) {
                    Ok(v) => v,
                    Err(e) => return ExecResult::Err(e),
                };
                let target = match self.lookup(name) {
                    Some(v) => v,
                    None => return ExecResult::Err(format!("Error: `{}` is not declared", name)),
                };

                match (target, idx_val) {
                    (Value::List(mut items), Value::Number(n)) => {
                        let i = n as usize;
                        if i >= items.len() {
                            return ExecResult::Err(format!(
                                "Error: index {} out of bounds (length {})", i, items.len()
                            ));
                        }
                        items[i] = new_val;
                        match self.assign(name, Value::List(items)) {
                            Ok(()) => ExecResult::Ok,
                            Err(e) => ExecResult::Err(e),
                        }
                    }
                    (Value::Map(mut map), Value::String(key)) => {
                        map.insert(key, new_val);
                        match self.assign(name, Value::Map(map)) {
                            Ok(()) => ExecResult::Ok,
                            Err(e) => ExecResult::Err(e),
                        }
                    }
                    _ => ExecResult::Err("Error: invalid index assignment target".to_string()),
                }
            }

            Stmt::MemberAssign { name, field, value } => {
                let new_val = match self.evaluate_expression(value) {
                    Ok(v) => v,
                    Err(e) => return ExecResult::Err(e),
                };
                let target = match self.lookup(name) {
                    Some(v) => v,
                    None => return ExecResult::Err(format!("Error: `{}` is not declared", name)),
                };
                match target {
                    Value::Object { type_name, mut fields } => {
                        fields.insert(field.clone(), new_val);
                        match self.assign(name, Value::Object { type_name, fields }) {
                            Ok(()) => ExecResult::Ok,
                            Err(e) => ExecResult::Err(e),
                        }
                    }
                    _ => ExecResult::Err(format!(
                        "Error: `{}` is not an object and cannot have fields set", name
                    )),
                }
            }

            Stmt::Say { exprs } => {
                let mut out = std::string::String::new();
                for (i, expr) in exprs.iter().enumerate() {
                    match self.evaluate_expression(expr) {
                        Ok(val) => {
                            if i > 0 { out.push(' '); }
                            out.push_str(&val.to_string());
                        }
                        Err(e) => return ExecResult::Err(e),
                    }
                }
                println!("{}", out);
                ExecResult::Ok
            }

            Stmt::Expr(expr) => match self.evaluate_expression(expr) {
                Ok(_) => ExecResult::Ok,
                Err(e) => ExecResult::Err(e),
            },

            Stmt::Block(stmts) => {
                self.environments.push(HashMap::new());
                let result = self.exec_block(stmts);
                self.environments.pop();
                result
            }

            Stmt::IfStmt { condition, then_branch, elif_branches, else_branch } => {
                match self.evaluate_expression(condition) {
                    Err(e) => return ExecResult::Err(e),
                    Ok(val) => {
                        if is_truthy(&val) {
                            return self.exec_stmt(then_branch);
                        }
                        for (elif_cond, elif_body) in elif_branches {
                            match self.evaluate_expression(elif_cond) {
                                Err(e) => return ExecResult::Err(e),
                                Ok(v) if is_truthy(&v) => return self.exec_stmt(elif_body),
                                _ => {}
                            }
                        }
                        if let Some(eb) = else_branch {
                            return self.exec_stmt(eb);
                        }
                    }
                }
                ExecResult::Ok
            }

            Stmt::WhileStmt { condition, body } => {
                loop {
                    match self.evaluate_expression(condition) {
                        Err(e) => return ExecResult::Err(e),
                        Ok(val) => { if !is_truthy(&val) { break; } }
                    }
                    match self.exec_stmt(body) {
                        ExecResult::Signal(Signal::Stop) => break,
                        ExecResult::Signal(Signal::Skip) => continue,
                        ExecResult::Ok => {}
                        other => return other,
                    }
                }
                ExecResult::Ok
            }

            Stmt::ForStmt { var, iterable, body } => {
                let list = match self.evaluate_expression(iterable) {
                    Ok(Value::List(l)) => l,
                    Ok(Value::String(s)) => {
                        s.chars().map(|c| Value::String(c.to_string())).collect()
                    }
                    Ok(other) => {
                        return ExecResult::Err(format!("Error: cannot iterate over {}", other));
                    }
                    Err(e) => return ExecResult::Err(e),
                };

                'outer: for item in list {
                    self.environments.push(HashMap::new());
                    self.environments.last_mut().unwrap()
                        .insert(var.clone(), (item, false));
                    let result = self.exec_stmt(body);
                    self.environments.pop();
                    match result {
                        ExecResult::Signal(Signal::Stop) => break 'outer,
                        ExecResult::Signal(Signal::Skip) => continue 'outer,
                        ExecResult::Ok => {}
                        other => return other,
                    }
                }
                ExecResult::Ok
            }

            Stmt::Stop => ExecResult::Signal(Signal::Stop),
            Stmt::Skip => ExecResult::Signal(Signal::Skip),

            Stmt::FnDecl { name, params, body } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                match self.declare(name.clone(), func, false) {
                    Ok(()) => ExecResult::Ok,
                    Err(e) => ExecResult::Err(e),
                }
            }

            Stmt::Give(expr) => match self.evaluate_expression(expr) {
                Ok(val) => ExecResult::Signal(Signal::Give(val)),
                Err(e) => ExecResult::Err(e),
            },

            Stmt::TypeDecl { name, methods } => {
                let mut method_map = HashMap::new();
                for method in methods {
                    if let Stmt::FnDecl { name: method_name, params, body } = method {
                        method_map.insert(method_name.clone(), (params.clone(), *body.clone()));
                    }
                }
                let type_val = Value::TypeDef {
                    name: name.clone(),
                    methods: method_map,
                };
                match self.declare(name.clone(), type_val, false) {
                    Ok(()) => ExecResult::Ok,
                    Err(e) => ExecResult::Err(e),
                }
            }

            Stmt::Use(path) => {
                match std::fs::read_to_string(path) {
                    Err(e) => ExecResult::Err(format!("Error loading module '{}': {}", path, e)),
                    Ok(source) => {
                        let mut lexer = crate::lexer::Lexer::new(&source);
                        let tokens = match lexer.tokenize() {
                            Ok(t) => t,
                            Err(e) => return ExecResult::Err(e),
                        };
                        let mut parser = crate::parser::Parser::new(tokens);
                        let program = match parser.parse() {
                            Ok(p) => p,
                            Err(e) => return ExecResult::Err(e),
                        };
                        for stmt in &program.statements {
                            let r = self.exec_stmt(stmt);
                            if !matches!(r, ExecResult::Ok) {
                                return r;
                            }
                        }
                        ExecResult::Ok
                    }
                }
            }

            Stmt::TryCatch { try_block, error_var, catch_block } => {
                let try_result = self.exec_stmt(try_block);
                let caught_val = match try_result {
                    ExecResult::Signal(Signal::Throw(err_val)) => Some(err_val),
                    ExecResult::Err(ref e) if e.starts_with("__throw__:") => {
                        let msg = e.trim_start_matches("__throw__:").to_string();
                        Some(Value::String(msg))
                    }
                    other => {
                        // Not a throw — propagate normally
                        return other;
                    }
                };
                if let Some(err_val) = caught_val {
                    self.environments.push(HashMap::new());
                    self.environments.last_mut().unwrap()
                        .insert(error_var.clone(), (err_val, false));
                    let result = self.exec_stmt(catch_block);
                    self.environments.pop();
                    result
                } else {
                    ExecResult::Ok
                }
            }

            Stmt::Throw(expr) => match self.evaluate_expression(expr) {
                Ok(val) => ExecResult::Signal(Signal::Throw(val)),
                Err(e) => ExecResult::Err(e),
            },

            Stmt::MatchStmt { target, cases, else_branch } => {
                let target_val = match self.evaluate_expression(target) {
                    Ok(v) => v,
                    Err(e) => return ExecResult::Err(e),
                };

                for (case_expr, case_body) in cases {
                    let case_val = match self.evaluate_expression(case_expr) {
                        Ok(v) => v,
                        Err(e) => return ExecResult::Err(e),
                    };
                    if values_equal(&target_val, &case_val) {
                        return self.exec_stmt(case_body);
                    }
                }

                if let Some(eb) = else_branch {
                    return self.exec_stmt(eb);
                }

                ExecResult::Ok
            }
        }
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> ExecResult {
        for stmt in stmts {
            let r = self.exec_stmt(stmt);
            if !matches!(r, ExecResult::Ok) {
                return r;
            }
        }
        ExecResult::Ok
    }

    // ── Expression evaluation ────────────────────────────────────────────────

    fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),

            Expr::Ident(name) => {
                if name == "nothing" {
                    return Ok(Value::Nothing);
                }
                self.lookup(name)
                    .ok_or_else(|| format!("Error: `{}` is not declared", name))
            }

            Expr::List(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.evaluate_expression(e)?);
                }
                Ok(Value::List(vals))
            }

            Expr::Map(elements) => {
                let mut map = HashMap::new();
                for (k, v) in elements {
                    let key_str = match self.evaluate_expression(k)? {
                        Value::String(s) => s,
                        _ => return Err("Map keys must be strings".to_string()),
                    };
                    let val = self.evaluate_expression(v)?;
                    map.insert(key_str, val);
                }
                Ok(Value::Map(map))
            }

            Expr::Tuple(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.evaluate_expression(e)?);
                }
                Ok(Value::Tuple(vals))
            }

            Expr::Index { target, index } => {
                let target_val = self.evaluate_expression(target)?;
                let index_val = self.evaluate_expression(index)?;
                self.eval_index(target_val, index_val)
            }

            Expr::MemberAccess { object, field } => {
                let obj_val = self.evaluate_expression(object)?;
                self.eval_member_access(obj_val, field)
            }

            Expr::MethodCall { object, method, args } => {
                // Evaluate args first
                let mut arg_vals = Vec::new();
                for a in args {
                    arg_vals.push(self.evaluate_expression(a)?);
                }
                let obj_val = self.evaluate_expression(object)?;
                self.eval_method_call(obj_val, method, arg_vals)
            }

            Expr::UnaryNot(operand) => {
                let val = self.evaluate_expression(operand)?;
                Ok(Value::Number(if is_truthy(&val) { 0.0 } else { 1.0 }))
            }

            Expr::Call { callee, args } => {
                let mut arg_vals = Vec::new();
                for a in args {
                    arg_vals.push(self.evaluate_expression(a)?);
                }
                let func_val = self.lookup(callee)
                    .ok_or_else(|| format!("Error: `{}` is not defined", callee))?;
                self.call_value(callee, func_val, arg_vals)
            }

            Expr::BinaryOp(left, op, right) => {
                // Short-circuit for And/Or
                match op {
                    BinaryOp::And => {
                        let l = self.evaluate_expression(left)?;
                        if !is_truthy(&l) {
                            return Ok(Value::Number(0.0));
                        }
                        let r = self.evaluate_expression(right)?;
                        return Ok(Value::Number(if is_truthy(&r) { 1.0 } else { 0.0 }));
                    }
                    BinaryOp::Or => {
                        let l = self.evaluate_expression(left)?;
                        if is_truthy(&l) {
                            return Ok(Value::Number(1.0));
                        }
                        let r = self.evaluate_expression(right)?;
                        return Ok(Value::Number(if is_truthy(&r) { 1.0 } else { 0.0 }));
                    }
                    _ => {}
                }
                let l = self.evaluate_expression(left)?;
                let r = self.evaluate_expression(right)?;
                eval_binary(l, *op, r)
            }
        }
    }

    // ── Indexing ─────────────────────────────────────────────────────────────

    fn eval_index(&self, target: Value, index: Value) -> Result<Value, String> {
        match (target, index) {
            (Value::List(items), Value::Number(n)) => {
                let i = n as usize;
                items.into_iter().nth(i)
                    .ok_or_else(|| format!("Error: index {} out of bounds", i))
            }
            (Value::Map(map), Value::String(key)) => {
                map.get(&key)
                    .cloned()
                    .ok_or_else(|| format!("Error: key '{}' not found in map", key))
            }
            (Value::Tuple(items), Value::Number(n)) => {
                let i = n as usize;
                items.into_iter().nth(i)
                    .ok_or_else(|| format!("Error: tuple index {} out of bounds", i))
            }
            (Value::String(s), Value::Number(n)) => {
                let i = n as usize;
                s.chars().nth(i)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or_else(|| format!("Error: string index {} out of bounds", i))
            }
            _ => Err("Error: unsupported index operation".to_string()),
        }
    }

    // ── Member access ─────────────────────────────────────────────────────────

    fn eval_member_access(&self, obj: Value, field: &str) -> Result<Value, String> {
        match obj {
            Value::Object { fields, .. } => {
                fields.get(field)
                    .cloned()
                    .ok_or_else(|| format!("Error: field '{}' not found", field))
            }
            Value::Map(map) => {
                map.get(field)
                    .cloned()
                    .ok_or_else(|| format!("Error: key '{}' not found in map", field))
            }
            _ => Err(format!("Error: cannot access field '{}' on this value", field)),
        }
    }

    // ── Method calls ─────────────────────────────────────────────────────────

    fn eval_method_call(&mut self, obj: Value, method: &str, args: Vec<Value>) -> Result<Value, String> {
        // Built-in methods for List, Map, String
        match &obj {
            Value::List(items) => {
                match method {
                    "len" => return Ok(Value::Number(items.len() as f64)),
                    "first" => return items.first().cloned()
                        .ok_or_else(|| "Error: list is empty".to_string()),
                    "last" => return items.last().cloned()
                        .ok_or_else(|| "Error: list is empty".to_string()),
                    "push" => {
                        if args.len() != 1 {
                            return Err("push() takes 1 argument".to_string());
                        }
                        let mut new_list = items.clone();
                        new_list.push(args.into_iter().next().unwrap());
                        return Ok(Value::List(new_list));
                    }
                    "pop" => {
                        let mut new_list = items.clone();
                        new_list.pop();
                        return Ok(Value::List(new_list));
                    }
                    "contains" => {
                        if args.len() != 1 { return Err("contains() takes 1 argument".to_string()); }
                        let needle = &args[0];
                        let found = items.iter().any(|v| values_equal(v, needle));
                        return Ok(Value::Number(if found { 1.0 } else { 0.0 }));
                    }
                    "join" => {
                        if args.len() != 1 { return Err("join() takes 1 argument".to_string()); }
                        let sep = match &args[0] {
                            Value::String(s) => s.clone(),
                            _ => return Err("join() separator must be a string".to_string()),
                        };
                        let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                        return Ok(Value::String(parts.join(&sep)));
                    }
                    "reverse" => {
                        let mut new_list = items.clone();
                        new_list.reverse();
                        return Ok(Value::List(new_list));
                    }
                    _ => {}
                }
            }
            Value::Map(map) => {
                match method {
                    "len" => return Ok(Value::Number(map.len() as f64)),
                    "keys" => {
                        let mut keys: Vec<Value> = map.keys()
                            .map(|k| Value::String(k.clone()))
                            .collect();
                        keys.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
                        return Ok(Value::List(keys));
                    }
                    "values" => {
                        let vals: Vec<Value> = map.values().cloned().collect();
                        return Ok(Value::List(vals));
                    }
                    "has" => {
                        if args.len() != 1 { return Err("has() takes 1 argument".to_string()); }
                        let key = match &args[0] {
                            Value::String(s) => s.clone(),
                            _ => return Err("has() key must be a string".to_string()),
                        };
                        return Ok(Value::Number(if map.contains_key(&key) { 1.0 } else { 0.0 }));
                    }
                    _ => {}
                }
            }
            Value::String(s) => {
                match method {
                    "len" => return Ok(Value::Number(s.chars().count() as f64)),
                    "upper" => return Ok(Value::String(s.to_uppercase())),
                    "lower" => return Ok(Value::String(s.to_lowercase())),
                    "trim" => return Ok(Value::String(s.trim().to_string())),
                    "contains" => {
                        if args.len() != 1 { return Err("contains() takes 1 argument".to_string()); }
                        let sub = match &args[0] {
                            Value::String(sub) => sub.clone(),
                            _ => return Err("contains() argument must be a string".to_string()),
                        };
                        return Ok(Value::Number(if s.contains(&sub as &str) { 1.0 } else { 0.0 }));
                    }
                    "starts_with" => {
                        if args.len() != 1 { return Err("starts_with() takes 1 argument".to_string()); }
                        let prefix = match &args[0] { Value::String(p) => p.clone(), _ => return Err("argument must be a string".to_string()) };
                        return Ok(Value::Number(if s.starts_with(&prefix as &str) { 1.0 } else { 0.0 }));
                    }
                    "ends_with" => {
                        if args.len() != 1 { return Err("ends_with() takes 1 argument".to_string()); }
                        let suffix = match &args[0] { Value::String(p) => p.clone(), _ => return Err("argument must be a string".to_string()) };
                        return Ok(Value::Number(if s.ends_with(&suffix as &str) { 1.0 } else { 0.0 }));
                    }
                    "split" => {
                        if args.len() != 1 { return Err("split() takes 1 argument".to_string()); }
                        let sep = match &args[0] { Value::String(p) => p.clone(), _ => return Err("split separator must be a string".to_string()) };
                        let parts: Vec<Value> = s.split(&sep as &str).map(|p| Value::String(p.to_string())).collect();
                        return Ok(Value::List(parts));
                    }
                    _ => {}
                }
            }
            Value::Tuple(items) => {
                if method == "len" {
                    return Ok(Value::Number(items.len() as f64));
                }
            }
            _ => {}
        }

        // User-defined type methods
        match &obj {
            Value::Object { type_name, fields } => {
                let type_name = type_name.clone();
                let obj_fields = fields.clone();

                let type_val = self.lookup(&type_name)
                    .ok_or_else(|| format!("Error: type `{}` is not defined", type_name))?;

                if let Value::TypeDef { methods, .. } = type_val {
                    if let Some((params, body)) = methods.get(method).cloned() {
                        if args.len() != params.len() {
                            return Err(format!(
                                "Error: {}.{}() expects {} argument(s), got {}",
                                type_name, method, params.len(), args.len()
                            ));
                        }

                        let global_env = self.environments[0].clone();
                        let saved_envs = std::mem::replace(
                            &mut self.environments,
                            vec![global_env],
                        );

                        // Bind self + params
                        let mut call_env = HashMap::new();
                        call_env.insert(
                            "self".to_string(),
                            (Value::Object { type_name: type_name.clone(), fields: obj_fields }, false),
                        );
                        for (param, val) in params.iter().zip(args.into_iter()) {
                            call_env.insert(param.clone(), (val, false));
                        }
                        self.environments.push(call_env);

                        let result = self.exec_stmt(&body);
                        self.environments = saved_envs;

                        return match result {
                            ExecResult::Ok => Ok(Value::Nothing),
                            ExecResult::Signal(Signal::Give(v)) => Ok(v),
                            ExecResult::Err(e) => Err(e),
                            _ => Err("Error: unexpected control flow in method".to_string()),
                        };
                    }
                    return Err(format!("Error: `{}` has no method `{}`", type_name, method));
                }

                Err(format!("Error: `{}` is not a valid type", type_name))
            }
            _ => Err(format!("Error: `.{}()` is not a valid method on this value", method)),
        }
    }

    // ── Function/type call ────────────────────────────────────────────────────

    fn call_value(&mut self, callee: &str, func_val: Value, arg_vals: Vec<Value>) -> Result<Value, String> {
        match func_val {
            Value::Function { params, body } => {
                if arg_vals.len() != params.len() {
                    return Err(format!(
                        "Error: `{}` expects {} argument(s), got {}",
                        callee, params.len(), arg_vals.len()
                    ));
                }
                let global_env = self.environments[0].clone();
                let saved_envs = std::mem::replace(
                    &mut self.environments,
                    vec![global_env],
                );
                let mut call_env = HashMap::new();
                for (param, val) in params.iter().zip(arg_vals.into_iter()) {
                    call_env.insert(param.clone(), (val, false));
                }
                self.environments.push(call_env);
                let result = self.exec_stmt(&body);
                self.environments = saved_envs;
                match result {
                    ExecResult::Ok => Ok(Value::Nothing),
                    ExecResult::Signal(Signal::Give(v)) => Ok(v),
                    ExecResult::Err(e) => Err(e),
                    ExecResult::Signal(Signal::Stop) => Err("'stop' used outside loop".to_string()),
                    ExecResult::Signal(Signal::Skip) => Err("'skip' used outside loop".to_string()),
                    // Re-wrap Throw as a special error so outer try/catch can catch it
                    ExecResult::Signal(Signal::Throw(v)) => Err(format!("__throw__:{}", v)),
                }
            }

            Value::TypeDef { name: type_name, methods } => {
                // Construct a new object and call init
                let new_obj = Value::Object {
                    type_name: type_name.clone(),
                    fields: HashMap::new(),
                };

                if let Some((params, body)) = methods.get("init").cloned() {
                    if arg_vals.len() != params.len() {
                        return Err(format!(
                            "Error: `{}`.init() expects {} argument(s), got {}",
                            type_name, params.len(), arg_vals.len()
                        ));
                    }

                    let global_env = self.environments[0].clone();
                    let saved_envs = std::mem::replace(
                        &mut self.environments,
                        vec![global_env],
                    );

                    let mut call_env = HashMap::new();
                    call_env.insert("self".to_string(), (new_obj, false));
                    for (param, val) in params.iter().zip(arg_vals.into_iter()) {
                        call_env.insert(param.clone(), (val, false));
                    }
                    self.environments.push(call_env);

                    let result = self.exec_stmt(&body);

                    // Extract the (possibly modified) self
                    let self_val = self.lookup("self").unwrap_or(Value::Nothing);
                    self.environments = saved_envs;

                    match result {
                        ExecResult::Ok | ExecResult::Signal(Signal::Give(_)) => Ok(self_val),
                        ExecResult::Err(e) => Err(e),
                        ExecResult::Signal(Signal::Throw(v)) => Err(format!("Uncaught error in constructor: {}", v)),
                        _ => Ok(self_val),
                    }
                } else if arg_vals.is_empty() {
                    // No init — return empty object
                    Ok(new_obj)
                } else {
                    Err(format!("Error: `{}` has no `init` method but was called with arguments", type_name))
                }
            }

            other => Err(format!("Error: `{}` is not callable (got {})", callee, other)),
        }
    }
}

// ── Binary operations ─────────────────────────────────────────────────────────

fn eval_binary(l: Value, op: BinaryOp, r: Value) -> Result<Value, String> {
    match (l, r) {
        (Value::Number(ln), Value::Number(rn)) => match op {
            BinaryOp::Add   => Ok(Value::Number(ln + rn)),
            BinaryOp::Sub   => Ok(Value::Number(ln - rn)),
            BinaryOp::Mul   => Ok(Value::Number(ln * rn)),
            BinaryOp::Div   => {
                if rn == 0.0 { Err("Error: division by zero".to_string()) }
                else { Ok(Value::Number(ln / rn)) }
            }
            BinaryOp::Eq    => Ok(Value::Number(if ln == rn { 1.0 } else { 0.0 })),
            BinaryOp::NotEq => Ok(Value::Number(if ln != rn { 1.0 } else { 0.0 })),
            BinaryOp::Lt    => Ok(Value::Number(if ln <  rn { 1.0 } else { 0.0 })),
            BinaryOp::Gt    => Ok(Value::Number(if ln >  rn { 1.0 } else { 0.0 })),
            BinaryOp::LtEq  => Ok(Value::Number(if ln <= rn { 1.0 } else { 0.0 })),
            BinaryOp::GtEq  => Ok(Value::Number(if ln >= rn { 1.0 } else { 0.0 })),
            BinaryOp::And | BinaryOp::Or => unreachable!("handled above"),
        },
        (Value::String(ls), Value::String(rs)) => match op {
            BinaryOp::Add   => Ok(Value::String(format!("{}{}", ls, rs))),
            BinaryOp::Eq    => Ok(Value::Number(if ls == rs { 1.0 } else { 0.0 })),
            BinaryOp::NotEq => Ok(Value::Number(if ls != rs { 1.0 } else { 0.0 })),
            _ => Err("Error: unsupported operation on strings".to_string()),
        },
        (Value::String(ls), r) => match op {
            BinaryOp::Add => Ok(Value::String(format!("{}{}", ls, r))),
            _ => Err("Error: unsupported operation".to_string()),
        },
        (l, Value::String(rs)) => match op {
            BinaryOp::Add => Ok(Value::String(format!("{}{}", l, rs))),
            _ => Err("Error: unsupported operation".to_string()),
        },
        _ => Err("Error: type mismatch in binary operation".to_string()),
    }
}
