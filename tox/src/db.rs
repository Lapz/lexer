use errors::FileId;
use parser::FilesExt;
use std::default::Default;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::Arc;
#[salsa::database(
    semant::HirDatabaseStorage,
    semant::InternDatabaseStorage,
    parser::ParseDatabaseStorage
)]
#[derive(Debug, Default)]
pub struct DatabaseImpl {
    runtime: salsa::Runtime<DatabaseImpl>,
    files: errors::Files<Arc<str>>,
}

pub(crate) trait Diagnostics {
    fn emit(&self) -> io::Result<()>;
}

impl Diagnostics for DatabaseImpl {
    fn emit(&self) -> io::Result<()> {
        Ok(())
    }
}

impl FilesExt for DatabaseImpl {
    fn source(&self, file: FileId) -> &Arc<str> {
        self.files.source(file)
    }

    fn load_file(&mut self, path: &PathBuf) -> FileId {
        let source = read_file(path).expect("Couldn't read a file");
        self.files.add(path, source.into())
    }
}

impl salsa::Database for DatabaseImpl {
    fn salsa_runtime(&self) -> &salsa::Runtime<DatabaseImpl> {
        &self.runtime
    }

    fn salsa_runtime_mut(&mut self) -> &mut salsa::Runtime<DatabaseImpl> {
        &mut self.runtime
    }
}

fn read_file(name: &PathBuf) -> io::Result<String> {
    let mut file = File::open(name)?;

    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}
