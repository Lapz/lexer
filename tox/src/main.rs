mod cli;
mod db;

use crate::cli::{Cli, Commands};
use crate::db::DatabaseImpl;

use parser::{dump_debug, FilesExt, ParseDatabase, Parser};

use semant::HirDatabase;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use structopt::StructOpt as _;
use syntax::Lexer;

pub type ParseResult<T> = Result<T, ()>;

pub fn parse<W: std::io::Write>(files: &[PathBuf], out: &mut W) -> std::io::Result<()> {
    let mut db = DatabaseImpl::default();

    for file in files {
        let handle = db.load_file(file);

        let program = db.parse(handle);
    }

    // for source in
    // let mut files = errors::Files::new();
    // let file_id = files.add("testing", source.into());
    // let reporter = errors::Reporter::new(files, file_id);
    // let mut lexer = Lexer::new(source, reporter.clone());
    // let mut parser = Parser::new(lexer.lex().into_iter(), reporter.clone(), source);
    // let source_file = parser.parse_program();

    // reporter.emit()?;
    // // write!(out, "{}", source_file.syntax().text())?;
    // write!(out, "{}", dump_debug(&source_file))?;

    // let program = if reporter.has_errors() {
    //     reporter.emit()?;
    //     return Ok(());
    // } else {
    //     db.lower_ast(source_file)
    // };

    // db.resolve_program(program.clone(), reporter.clone());

    // reporter.emit()?;

    Ok(())
}

pub fn lex<W: std::io::Write>(sources: &[PathBuf], out: &mut W) -> std::io::Result<()> {
    unimplemented!();
    // let mut files = errors::Files::new();
    // let file_id = files.add("testing", source.into());
    // let reporter = errors::Reporter::new(files, file_id);
    // let tokens = Lexer::new(source, reporter.clone()).lex();
    // reporter.emit()?;
    // for token in tokens {
    //     writeln!(out, "{:#?}", token)?
    // }

    Ok(())
}

pub fn run<W: std::io::Write>(
    command: Commands,
    sources: &[PathBuf],
    output: &mut W,
) -> std::io::Result<()> {
    match command {
        Commands::Lex => lex(sources, output),
        Commands::Parse => parse(sources, output),
    }
}

pub fn read_file(name: &std::path::PathBuf) -> std::io::Result<String> {
    let mut file = File::open(name)?;

    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn read_stdin() -> std::io::Result<String> {
    let mut input = String::with_capacity(1024);

    let _ = io::stdout().flush();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

pub fn get_input(sources: Vec<std::path::PathBuf>) -> std::io::Result<Vec<String>> {
    if sources.is_empty() {
        Ok(vec![read_stdin()?])
    } else {
        Ok(sources
            .iter()
            .map(|source| read_file(source).expect("Coudln't read a file"))
            .collect())
    }
}

fn main() -> std::io::Result<()> {
    let opt = Cli::from_args();

    let input = opt.source;
    let output = opt.output;

    let command = if opt.lex {
        Commands::Lex
    } else {
        Commands::Parse
    };

    if let Some(output) = output {
        run(command, &input, &mut File::open(output)?)?
    } else {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        run(command, &input, &mut stdout)?
    }

    // let input = "fn main() { 1 + (1+10);}";
    // let mut lexer = Lexer::new(input);
    // let mut parser = Parser::new(lexer.lex().into_iter(), input);
    // let file = parser.parse_program();

    // println!("{:#?}", file);

    // let func = file.functions().nth(0).unwrap();

    // if let Some(params) = func.param_list() {
    //     for param in params.params() {
    //         println!("{:?}", param.pat());
    //         println!("{:?}", param.ascribed_type());
    //     }
    // }

    // println!("{:?}", func.visibility());

    // for token in lexer.lex() {
    //     println!(
    //         "{:?},{:?}",
    //         SmolStr::new(
    //             &input[token.start.absolute as usize
    //                 ..(token.value.len + token.start.absolute) as usize]
    //         ),
    //         token.value.kind
    //     )
    // }

    Ok(())
}
