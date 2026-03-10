use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::rc::Rc;

use wb_ast::{BinaryOp, Expr, Literal, Stmt, UnaryOp};
use wb_diagnostics::Diagnostic;

pub struct LoadedModule {
    pub statements: Vec<Stmt>,
    pub base_dir: PathBuf,
}

pub trait ModuleLoader {
    fn load(&mut self, name: &str) -> Result<Option<LoadedModule>, Diagnostic>;
    fn enter(&mut self, base_dir: PathBuf);
    fn exit(&mut self);
}

struct NullLoader;

impl ModuleLoader for NullLoader {
    fn load(&mut self, _name: &str) -> Result<Option<LoadedModule>, Diagnostic> {
        Err(Diagnostic::new("Module loader belum diinisialisasi"))
    }

    fn enter(&mut self, _base_dir: PathBuf) {}

    fn exit(&mut self) {}
}

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    Array(Vec<Value>),
    Function(Function),
    NativeFunction(NativeFunction),
}

#[derive(Clone)]
pub struct Function {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
    closure: EnvRef,
}

#[derive(Clone, Copy)]
pub enum Arity {
    Exact(usize),
    Variadic,
}

#[derive(Clone)]
pub struct NativeFunction {
    name: &'static str,
    arity: Arity,
    func: fn(Vec<Value>) -> Result<Value, Diagnostic>,
}

type EnvRef = Rc<RefCell<Environment>>;

struct Environment {
    values: HashMap<String, Value>,
    parent: Option<EnvRef>,
}

impl Environment {
    fn new(parent: Option<EnvRef>) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            parent,
        }))
    }

    fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    fn assign(&mut self, name: &str, value: Value) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return true;
        }
        if let Some(parent) = self.parent.clone() {
            return parent.borrow_mut().assign(name, value);
        }
        false
    }

    fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            return Some(value.clone());
        }
        if let Some(parent) = self.parent.clone() {
            return parent.borrow().get(name);
        }
        None
    }
}

enum ExecSignal {
    None,
    Return(Value),
    Break,
    Continue,
}

pub struct Interpreter {
    env: EnvRef,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Environment::new(None);
        install_builtins(&env);
        Self { env }
    }

    pub fn eval(&mut self, statements: &[Stmt]) -> Result<Value, Diagnostic> {
        let mut loader = NullLoader;
        self.eval_with_loader(statements, &mut loader)
    }

    pub fn eval_with_loader(
        &mut self,
        statements: &[Stmt],
        loader: &mut dyn ModuleLoader,
    ) -> Result<Value, Diagnostic> {
        for stmt in statements {
            match self.execute(stmt, loader)? {
                ExecSignal::None => {}
                ExecSignal::Return(_) => {
                    return Err(Diagnostic::new("'balikin' tidak boleh di luar fungsi"));
                }
                ExecSignal::Break => {
                    return Err(Diagnostic::new("'berhenti' tidak boleh di luar loop"));
                }
                ExecSignal::Continue => {
                    return Err(Diagnostic::new("'lanjut' tidak boleh di luar loop"));
                }
            }
        }
        Ok(Value::Nil)
    }

    fn execute(&mut self, stmt: &Stmt, loader: &mut dyn ModuleLoader) -> Result<ExecSignal, Diagnostic> {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval_expr(expr, loader)?;
                Ok(ExecSignal::None)
            }
            Stmt::Let { name, value } => {
                let value = self.eval_expr(value, loader)?;
                self.env.borrow_mut().define(name.clone(), value);
                Ok(ExecSignal::None)
            }
            Stmt::Assign { name, value } => {
                let value = self.eval_expr(value, loader)?;
                if !self.env.borrow_mut().assign(name, value) {
                    return Err(Diagnostic::new(format!(
                        "Variable '{}' belum dideklarasikan",
                        name
                    )));
                }
                Ok(ExecSignal::None)
            }
            Stmt::Block(statements) => {
                let env = Environment::new(Some(self.env.clone()));
                self.execute_block(statements, env, loader)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&self.eval_expr(condition, loader)?) {
                    let env = Environment::new(Some(self.env.clone()));
                    self.execute_block(then_branch, env, loader)
                } else if let Some(else_branch) = else_branch {
                    let env = Environment::new(Some(self.env.clone()));
                    self.execute_block(else_branch, env, loader)
                } else {
                    Ok(ExecSignal::None)
                }
            }
            Stmt::While { condition, body } => {
                loop {
                    let cond = self.eval_expr(condition, loader)?;
                    if !is_truthy(&cond) {
                        break;
                    }
                    let env = Environment::new(Some(self.env.clone()));
                    match self.execute_block(body, env, loader)? {
                        ExecSignal::None => {}
                        ExecSignal::Break => break,
                        ExecSignal::Continue => continue,
                        ExecSignal::Return(value) => return Ok(ExecSignal::Return(value)),
                    }
                }
                Ok(ExecSignal::None)
            }
            Stmt::ForEach { name, iterable, body } => {
                let iterable = self.eval_expr(iterable, loader)?;
                match iterable {
                    Value::Array(items) => {
                        for item in items {
                            let env = Environment::new(Some(self.env.clone()));
                            env.borrow_mut().define(name.clone(), item);
                            match self.execute_block(body, env, loader)? {
                                ExecSignal::None => {}
                                ExecSignal::Break => break,
                                ExecSignal::Continue => continue,
                                ExecSignal::Return(value) => return Ok(ExecSignal::Return(value)),
                            }
                        }
                        Ok(ExecSignal::None)
                    }
                    _ => Err(Diagnostic::new("'ulang' hanya bisa untuk array")),
                }
            }
            Stmt::Function { name, params, body } => {
                let func = Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.env.clone(),
                };
                self.env
                    .borrow_mut()
                    .define(name.clone(), Value::Function(func));
                Ok(ExecSignal::None)
            }
            Stmt::Return(expr) => {
                let value = match expr {
                    Some(expr) => self.eval_expr(expr, loader)?,
                    None => Value::Nil,
                };
                Ok(ExecSignal::Return(value))
            }
            Stmt::Break => Ok(ExecSignal::Break),
            Stmt::Continue => Ok(ExecSignal::Continue),
            Stmt::Import { module } => {
                let module_value = self.eval_expr(module, loader)?;
                let module_name = match module_value {
                    Value::String(name) => name,
                    _ => {
                        return Err(Diagnostic::new(
                            "butuh hanya menerima nama modul string",
                        ))
                    }
                };
                if let Some(module) = loader.load(&module_name)? {
                    loader.enter(module.base_dir.clone());
                    let result = self.eval_with_loader(&module.statements, loader);
                    loader.exit();
                    result?;
                }
                Ok(ExecSignal::None)
            }
            Stmt::Export { value } => {
                let _ = self.eval_expr(value, loader)?;
                Ok(ExecSignal::None)
            }
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        env: EnvRef,
        loader: &mut dyn ModuleLoader,
    ) -> Result<ExecSignal, Diagnostic> {
        let previous = self.env.clone();
        self.env = env;
        let mut result = ExecSignal::None;
        for stmt in statements {
            match self.execute(stmt, loader)? {
                ExecSignal::None => {}
                signal => {
                    result = signal;
                    break;
                }
            }
        }
        self.env = previous;
        Ok(result)
    }

    fn eval_expr(
        &mut self,
        expr: &Expr,
        loader: &mut dyn ModuleLoader,
    ) -> Result<Value, Diagnostic> {
        match expr {
            Expr::Literal(literal) => Ok(match literal {
                Literal::Number(n) => Value::Number(*n),
                Literal::String(s) => Value::String(s.clone()),
                Literal::Boolean(b) => Value::Boolean(*b),
                Literal::Nil => Value::Nil,
            }),
            Expr::Array(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.eval_expr(item, loader)?);
                }
                Ok(Value::Array(values))
            }
            Expr::Identifier(name) => self
                .env
                .borrow()
                .get(name)
                .ok_or_else(|| Diagnostic::new(format!("Variable '{}' tidak ditemukan", name))),
            Expr::Unary { op, expr } => {
                let value = self.eval_expr(expr, loader)?;
                match op {
                    UnaryOp::Negate => match value {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(Diagnostic::new("Operator '-' hanya untuk angka")),
                    },
                    UnaryOp::Not => Ok(Value::Boolean(!is_truthy(&value))),
                }
            }
            Expr::Binary { left, op, right } => {
                if matches!(op, BinaryOp::And) {
                    let left_value = self.eval_expr(left, loader)?;
                    if !is_truthy(&left_value) {
                        return Ok(Value::Boolean(false));
                    }
                    let right_value = self.eval_expr(right, loader)?;
                    return Ok(Value::Boolean(is_truthy(&right_value)));
                }
                if matches!(op, BinaryOp::Or) {
                    let left_value = self.eval_expr(left, loader)?;
                    if is_truthy(&left_value) {
                        return Ok(Value::Boolean(true));
                    }
                    let right_value = self.eval_expr(right, loader)?;
                    return Ok(Value::Boolean(is_truthy(&right_value)));
                }

                let left_value = self.eval_expr(left, loader)?;
                let right_value = self.eval_expr(right, loader)?;

                match op {
                    BinaryOp::Add => match (&left_value, &right_value) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                        _ => Ok(Value::String(format!(
                            "{}{}",
                            value_to_string(&left_value),
                            value_to_string(&right_value)
                        ))),
                    },
                    BinaryOp::Subtract => Ok(Value::Number(
                        expect_number(&left_value)? - expect_number(&right_value)?,
                    )),
                    BinaryOp::Multiply => Ok(Value::Number(
                        expect_number(&left_value)? * expect_number(&right_value)?,
                    )),
                    BinaryOp::Divide => Ok(Value::Number(
                        expect_number(&left_value)? / expect_number(&right_value)?,
                    )),
                    BinaryOp::Modulo => Ok(Value::Number(
                        expect_number(&left_value)? % expect_number(&right_value)?,
                    )),
                    BinaryOp::Equal => Ok(Value::Boolean(values_equal(&left_value, &right_value))),
                    BinaryOp::NotEqual => Ok(Value::Boolean(!values_equal(&left_value, &right_value))),
                    BinaryOp::Less => Ok(Value::Boolean(
                        expect_number(&left_value)? < expect_number(&right_value)?,
                    )),
                    BinaryOp::LessEqual => Ok(Value::Boolean(
                        expect_number(&left_value)? <= expect_number(&right_value)?,
                    )),
                    BinaryOp::Greater => Ok(Value::Boolean(
                        expect_number(&left_value)? > expect_number(&right_value)?,
                    )),
                    BinaryOp::GreaterEqual => Ok(Value::Boolean(
                        expect_number(&left_value)? >= expect_number(&right_value)?,
                    )),
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                }
            }
            Expr::Call { callee, args } => {
                let callee_value = self.eval_expr(callee, loader)?;
                let mut evaluated_args = Vec::with_capacity(args.len());
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg, loader)?);
                }

                match callee_value {
                    Value::Function(function) => self.call_function(function, evaluated_args, loader),
                    Value::NativeFunction(native) => {
                        check_arity(native.name, native.arity, evaluated_args.len())?;
                        (native.func)(evaluated_args)
                    }
                    _ => Err(Diagnostic::new("Hanya fungsi yang bisa dipanggil")),
                }
            }
            Expr::Index { target, index } => {
                let target_value = self.eval_expr(target, loader)?;
                let index_value = self.eval_expr(index, loader)?;
                match (target_value, index_value) {
                    (Value::Array(items), Value::Number(n)) => {
                        if n.fract() != 0.0 {
                            return Err(Diagnostic::new("Index array harus integer"));
                        }
                        let idx = n as isize;
                        if idx < 0 || idx as usize >= items.len() {
                            return Err(Diagnostic::new("Index array di luar batas"));
                        }
                        Ok(items[idx as usize].clone())
                    }
                    _ => Err(Diagnostic::new("Index hanya berlaku untuk array dengan angka")),
                }
            }
        }
    }

    fn call_function(
        &mut self,
        function: Function,
        args: Vec<Value>,
        loader: &mut dyn ModuleLoader,
    ) -> Result<Value, Diagnostic> {
        if function.params.len() != args.len() {
            return Err(Diagnostic::new(format!(
                "Fungsi '{}' butuh {} argumen", 
                function.name,
                function.params.len()
            )));
        }

        let env = Environment::new(Some(function.closure));
        for (param, arg) in function.params.iter().cloned().zip(args.into_iter()) {
            env.borrow_mut().define(param, arg);
        }

        let result = self.execute_block(&function.body, env, loader)?;
        match result {
            ExecSignal::Return(value) => Ok(value),
            ExecSignal::Break => Err(Diagnostic::new("'berhenti' tidak valid di dalam fungsi")),
            ExecSignal::Continue => Err(Diagnostic::new("'lanjut' tidak valid di dalam fungsi")),
            ExecSignal::None => Ok(Value::Nil),
        }
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Boolean(b) => *b,
        Value::Nil => false,
        _ => true,
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Nil, Value::Nil) => true,
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(a, b)| values_equal(a, b)),
        _ => false,
    }
}

fn expect_number(value: &Value) -> Result<f64, Diagnostic> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(Diagnostic::new("Diharapkan angka")),
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Number(n) => format_number(*n),
        Value::String(s) => s.clone(),
        Value::Boolean(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Nil => "nil".to_string(),
        Value::Array(items) => {
            let inner = items
                .iter()
                .map(value_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", inner)
        }
        Value::Function(func) => format!("<fun {}>", func.name),
        Value::NativeFunction(func) => format!("<native {}>", func.name),
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

fn check_arity(name: &str, arity: Arity, given: usize) -> Result<(), Diagnostic> {
    match arity {
        Arity::Exact(expected) if expected != given => Err(Diagnostic::new(format!(
            "Fungsi '{}' butuh {} argumen", name, expected
        ))),
        _ => Ok(()),
    }
}

fn install_builtins(env: &EnvRef) {
    let mut env = env.borrow_mut();
    env.define(
        "baka".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "baka",
            arity: Arity::Variadic,
            func: builtin_baka,
        }),
    );
    env.define(
        "bakaf".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "bakaf",
            arity: Arity::Variadic,
            func: builtin_bakaf,
        }),
    );
    env.define(
        "format".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "format",
            arity: Arity::Variadic,
            func: builtin_format,
        }),
    );
    env.define(
        "input".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "input",
            arity: Arity::Variadic,
            func: builtin_input,
        }),
    );
    env.define(
        "panjang".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "panjang",
            arity: Arity::Exact(1),
            func: builtin_panjang,
        }),
    );
    env.define(
        "tipe".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "tipe",
            arity: Arity::Exact(1),
            func: builtin_tipe,
        }),
    );
    env.define(
        "angka".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "angka",
            arity: Arity::Exact(1),
            func: builtin_angka,
        }),
    );
    env.define(
        "teks".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "teks",
            arity: Arity::Exact(1),
            func: builtin_teks,
        }),
    );
    env.define(
        "stdout".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "stdout",
            arity: Arity::Variadic,
            func: builtin_stdout,
        }),
    );
    env.define(
        "stderr".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "stderr",
            arity: Arity::Variadic,
            func: builtin_stderr,
        }),
    );
    env.define(
        "baca_file".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "baca_file",
            arity: Arity::Exact(1),
            func: builtin_baca_file,
        }),
    );
    env.define(
        "tulis_file".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "tulis_file",
            arity: Arity::Exact(2),
            func: builtin_tulis_file,
        }),
    );
    env.define(
        "append_file".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "append_file",
            arity: Arity::Exact(2),
            func: builtin_append_file,
        }),
    );
    env.define(
        "cwd".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "cwd",
            arity: Arity::Exact(0),
            func: builtin_cwd,
        }),
    );
    env.define(
        "env_get".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "env_get",
            arity: Arity::Exact(1),
            func: builtin_env_get,
        }),
    );
    env.define(
        "env_set".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "env_set",
            arity: Arity::Exact(2),
            func: builtin_env_set,
        }),
    );
    env.define(
        "sqrt".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "sqrt",
            arity: Arity::Exact(1),
            func: builtin_sqrt,
        }),
    );
    env.define(
        "sin".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "sin",
            arity: Arity::Exact(1),
            func: builtin_sin,
        }),
    );
    env.define(
        "cos".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "cos",
            arity: Arity::Exact(1),
            func: builtin_cos,
        }),
    );
    env.define(
        "tan".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "tan",
            arity: Arity::Exact(1),
            func: builtin_tan,
        }),
    );
    env.define(
        "pow".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "pow",
            arity: Arity::Exact(2),
            func: builtin_pow,
        }),
    );
    env.define(
        "abs".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "abs",
            arity: Arity::Exact(1),
            func: builtin_abs,
        }),
    );
    env.define(
        "floor".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "floor",
            arity: Arity::Exact(1),
            func: builtin_floor,
        }),
    );
    env.define(
        "ceil".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "ceil",
            arity: Arity::Exact(1),
            func: builtin_ceil,
        }),
    );
    env.define(
        "round".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "round",
            arity: Arity::Exact(1),
            func: builtin_round,
        }),
    );
    env.define(
        "regex_cocok".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "regex_cocok",
            arity: Arity::Exact(2),
            func: builtin_regex_cocok,
        }),
    );
    env.define(
        "regex_cari".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "regex_cari",
            arity: Arity::Exact(2),
            func: builtin_regex_cari,
        }),
    );
    env.define(
        "regex_ganti".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "regex_ganti",
            arity: Arity::Exact(3),
            func: builtin_regex_ganti,
        }),
    );
}

fn builtin_baka(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let output = args
        .iter()
        .map(value_to_string)
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", output);
    Ok(Value::Nil)
}

fn builtin_bakaf(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let formatted = format_args(args)?;
    println!("{}", formatted);
    Ok(Value::Nil)
}

fn builtin_format(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let formatted = format_args(args)?;
    Ok(Value::String(formatted))
}

fn builtin_input(args: Vec<Value>) -> Result<Value, Diagnostic> {
    if let Some(prompt) = args.first() {
        print!("{}", value_to_string(prompt));
        io::stdout().flush().ok();
    }
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_| Diagnostic::new("Gagal membaca input"))?;
    if input.ends_with('\n') {
        input.pop();
        if input.ends_with('\r') {
            input.pop();
        }
    }
    Ok(Value::String(input))
}

fn builtin_panjang(args: Vec<Value>) -> Result<Value, Diagnostic> {
    match args.into_iter().next() {
        Some(Value::String(s)) => Ok(Value::Number(s.chars().count() as f64)),
        Some(Value::Array(items)) => Ok(Value::Number(items.len() as f64)),
        _ => Err(Diagnostic::new("panjang() menerima string atau array")),
    }
}

fn builtin_tipe(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let value = args.into_iter().next().unwrap_or(Value::Nil);
    let type_name = match value {
        Value::Number(_) => "angka",
        Value::String(_) => "teks",
        Value::Boolean(_) => "boolean",
        Value::Nil => "nil",
        Value::Array(_) => "array",
        Value::Function(_) | Value::NativeFunction(_) => "fungsi",
    };
    Ok(Value::String(type_name.to_string()))
}

fn builtin_angka(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let value = args.into_iter().next().unwrap_or(Value::Nil);
    match value {
        Value::Number(n) => Ok(Value::Number(n)),
        Value::String(s) => s
            .trim()
            .parse::<f64>()
            .map(Value::Number)
            .map_err(|_| Diagnostic::new("Tidak bisa konversi string ke angka")),
        Value::Boolean(b) => Ok(Value::Number(if b { 1.0 } else { 0.0 })),
        Value::Nil => Ok(Value::Number(0.0)),
        _ => Err(Diagnostic::new("Tidak bisa konversi ke angka")),
    }
}

fn builtin_teks(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let value = args.into_iter().next().unwrap_or(Value::Nil);
    Ok(Value::String(value_to_string(&value)))
}

fn builtin_stdout(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let output = join_args(args);
    print!("{}", output);
    io::stdout().flush().ok();
    Ok(Value::Nil)
}

fn builtin_stderr(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let output = join_args(args);
    eprint!("{}", output);
    io::stderr().flush().ok();
    Ok(Value::Nil)
}

fn builtin_baca_file(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let path = expect_string(&args[0])?;
    let contents =
        fs::read_to_string(&path).map_err(|_| Diagnostic::new("Gagal membaca file"))?;
    Ok(Value::String(contents))
}

fn builtin_tulis_file(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let path = expect_string(&args[0])?;
    let data = value_to_string(&args[1]);
    fs::write(&path, data).map_err(|_| Diagnostic::new("Gagal menulis file"))?;
    Ok(Value::Nil)
}

fn builtin_append_file(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let path = expect_string(&args[0])?;
    let data = value_to_string(&args[1]);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|_| Diagnostic::new("Gagal membuka file"))?;
    file.write_all(data.as_bytes())
        .map_err(|_| Diagnostic::new("Gagal menulis file"))?;
    Ok(Value::Nil)
}

fn builtin_cwd(_args: Vec<Value>) -> Result<Value, Diagnostic> {
    let cwd = env::current_dir()
        .map_err(|_| Diagnostic::new("Gagal membaca working directory"))?;
    Ok(Value::String(cwd.to_string_lossy().to_string()))
}

fn builtin_env_get(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let key = expect_string(&args[0])?;
    match env::var(key) {
        Ok(value) => Ok(Value::String(value)),
        Err(_) => Ok(Value::Nil),
    }
}

fn builtin_env_set(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let key = expect_string(&args[0])?;
    let value = value_to_string(&args[1]);
    unsafe {
        env::set_var(key, value);
    }
    Ok(Value::Nil)
}

fn builtin_sqrt(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let n = expect_number(&args[0])?;
    Ok(Value::Number(n.sqrt()))
}

fn builtin_sin(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let n = expect_number(&args[0])?;
    Ok(Value::Number(n.sin()))
}

fn builtin_cos(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let n = expect_number(&args[0])?;
    Ok(Value::Number(n.cos()))
}

fn builtin_tan(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let n = expect_number(&args[0])?;
    Ok(Value::Number(n.tan()))
}

fn builtin_pow(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let base = expect_number(&args[0])?;
    let exp = expect_number(&args[1])?;
    Ok(Value::Number(base.powf(exp)))
}

fn builtin_abs(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let n = expect_number(&args[0])?;
    Ok(Value::Number(n.abs()))
}

fn builtin_floor(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let n = expect_number(&args[0])?;
    Ok(Value::Number(n.floor()))
}

fn builtin_ceil(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let n = expect_number(&args[0])?;
    Ok(Value::Number(n.ceil()))
}

fn builtin_round(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let n = expect_number(&args[0])?;
    Ok(Value::Number(n.round()))
}

fn builtin_regex_cocok(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let pattern = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    let re = Regex::new(&pattern).map_err(|_| Diagnostic::new("Regex tidak valid"))?;
    Ok(Value::Boolean(re.is_match(&text)))
}

fn builtin_regex_cari(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let pattern = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    let re = Regex::new(&pattern).map_err(|_| Diagnostic::new("Regex tidak valid"))?;
    match re.find(&text) {
        Some(m) => Ok(Value::String(m.as_str().to_string())),
        None => Ok(Value::Nil),
    }
}

fn builtin_regex_ganti(args: Vec<Value>) -> Result<Value, Diagnostic> {
    let pattern = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    let replacement = expect_string(&args[2])?;
    let re = Regex::new(&pattern).map_err(|_| Diagnostic::new("Regex tidak valid"))?;
    Ok(Value::String(re.replace_all(&text, replacement).to_string()))
}

fn format_args(args: Vec<Value>) -> Result<String, Diagnostic> {
    if args.is_empty() {
        return Ok(String::new());
    }
    let mut iter = args.into_iter();
    let format_value = iter.next().unwrap();
    let format_string = match format_value {
        Value::String(s) => s,
        other => value_to_string(&other),
    };
    let rest = iter.collect::<Vec<_>>();
    let final_args = if rest.len() == 1 {
        if let Value::Array(items) = rest[0].clone() {
            items
        } else {
            rest
        }
    } else {
        rest
    };
    Ok(apply_format(&format_string, &final_args))
}

fn apply_format(format_string: &str, args: &[Value]) -> String {
    let mut output = String::new();
    let mut chars = format_string.chars().peekable();
    let mut index = 0;

    while let Some(ch) = chars.next() {
        match ch {
            '{' => match chars.peek() {
                Some('{') => {
                    chars.next();
                    output.push('{');
                }
                Some('}') => {
                    chars.next();
                    if let Some(value) = args.get(index) {
                        output.push_str(&value_to_string(value));
                        index += 1;
                    } else {
                        output.push_str("{}");
                    }
                }
                _ => output.push('{'),
            },
            '}' => match chars.peek() {
                Some('}') => {
                    chars.next();
                    output.push('}');
                }
                _ => output.push('}'),
            },
            _ => output.push(ch),
        }
    }

    output
}

fn join_args(args: Vec<Value>) -> String {
    args.into_iter()
        .map(|value| value_to_string(&value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn expect_string(value: &Value) -> Result<String, Diagnostic> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => Err(Diagnostic::new("Diharapkan string")),
    }
}
