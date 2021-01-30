#[macro_use]
mod macros;
mod chunks;
mod codegen;
mod db;
mod native;
mod object;
mod value;
mod vm;

pub use db::{CodegenDatabase, CodegenDatabaseStorage};
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
