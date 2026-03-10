use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use wb_diagnostics::Diagnostic;
use wb_runtime::{LoadedModule, ModuleLoader};

pub struct Session {
    interpreter: wb_runtime::Interpreter,
    loader: CoreModuleLoader,
}

impl Session {
    pub fn new() -> Self {
        let base_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            interpreter: wb_runtime::Interpreter::new(),
            loader: CoreModuleLoader::new(base_dir),
        }
    }

    pub fn exec(&mut self, source: &str) -> Result<(), Diagnostic> {
        let tokens = wb_lexer::lex(source);
        let statements = wb_parser::parse(&tokens)?;
        self.interpreter
            .eval_with_loader(&statements, &mut self.loader)?;
        Ok(())
    }

    pub fn exec_file(&mut self, path: &Path) -> Result<(), Diagnostic> {
        let source = fs::read_to_string(path)
            .map_err(|_| Diagnostic::new("Gagal membaca file"))?;
        let base_dir = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        self.loader.set_base_dir(base_dir);
        self.exec(&source)?;
        Ok(())
    }
}

pub fn interpret(source: &str) -> Result<(), Diagnostic> {
    let mut session = Session::new();
    session.exec(source)
}

struct CoreModuleLoader {
    std_root: PathBuf,
    dir_stack: Vec<PathBuf>,
    loaded: HashSet<PathBuf>,
}

impl CoreModuleLoader {
    fn new(base_dir: PathBuf) -> Self {
        Self {
            std_root: stdlib_root(),
            dir_stack: vec![base_dir],
            loaded: HashSet::new(),
        }
    }

    fn set_base_dir(&mut self, base_dir: PathBuf) {
        self.dir_stack.clear();
        self.dir_stack.push(base_dir);
    }

    fn current_dir(&self) -> PathBuf {
        self.dir_stack
            .last()
            .cloned()
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn resolve_module_path(&self, name: &str) -> Result<PathBuf, Diagnostic> {
        if let Some(rest) = name.strip_prefix("wb:") {
            if rest.is_empty() {
                return Err(Diagnostic::new("Nama modul wb: kosong"));
            }
            let base = if rest == "std" {
                self.std_root.join("std")
            } else {
                self.std_root.join(rest)
            };
            return resolve_path(base);
        }

        let raw = Path::new(name);
        let path = if raw.is_absolute() {
            raw.to_path_buf()
        } else {
            self.current_dir().join(raw)
        };
        resolve_path(path)
    }
}

impl ModuleLoader for CoreModuleLoader {
    fn load(&mut self, name: &str) -> Result<Option<LoadedModule>, Diagnostic> {
        let path = self.resolve_module_path(name)?;
        let canonical = fs::canonicalize(&path)
            .map_err(|_| Diagnostic::new("Gagal membaca modul"))?;
        if self.loaded.contains(&canonical) {
            return Ok(None);
        }
        self.loaded.insert(canonical.clone());

        let source = fs::read_to_string(&path)
            .map_err(|_| Diagnostic::new("Gagal membaca modul"))?;
        let tokens = wb_lexer::lex(&source);
        let statements = wb_parser::parse(&tokens)?;
        let base_dir = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        Ok(Some(LoadedModule { statements, base_dir }))
    }

    fn enter(&mut self, base_dir: PathBuf) {
        self.dir_stack.push(base_dir);
    }

    fn exit(&mut self) {
        if self.dir_stack.len() > 1 {
            self.dir_stack.pop();
        }
    }
}

fn resolve_path(base: PathBuf) -> Result<PathBuf, Diagnostic> {
    if base.is_dir() {
        let init = base.join("__init__.wb");
        if init.exists() {
            return Ok(init);
        }
    }
    if base.extension().is_none() {
        let with_ext = base.with_extension("wb");
        if with_ext.exists() {
            return Ok(with_ext);
        }
    }
    if base.exists() {
        return Ok(base);
    }
    Err(Diagnostic::new("Modul tidak ditemukan"))
}

fn stdlib_root() -> PathBuf {
    #[cfg(windows)]
    {
        env::var("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\"))
            .join(".wb")
            .join("lib")
            .join("wb")
    }
    #[cfg(not(windows))]
    {
        env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".wb")
            .join("lib")
            .join("wb")
    }
}
