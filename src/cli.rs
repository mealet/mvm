use clap::{Command, arg};
use colored::Colorize;

pub fn cli() -> Command {
    Command::new("mvm")
        .about("Mealet's 64-bit virtual machine")
        .version(env!("CARGO_PKG_VERSION"))
        .help_template("{options}")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .about("run virtual machine with compiled program")
                .arg(
                    arg!(-m <MEMSIZE> "machine memory size in bytes")
                        .default_value("1024")
                        .required(false),
                )
                .arg(
                    arg!(-s <STACKSIZE> "stack size in bytes")
                        .default_value("128")
                        .required(false),
                )
                .arg(arg!(<PROGRAM> "path to program binary file")),
        )
        .subcommand(
            Command::new("compile")
                .arg(arg!(-d --debug "debug build of program"))
                .arg(arg!(<ASM> "assembly file path"))
                .arg_required_else_help(true),
        )
}

pub fn error(message: impl std::fmt::Display) {
    eprintln!("{} {}", "Error:".red().bold(), message);
}

pub fn vm_error(message: impl std::fmt::Display) {
    eprintln!("{} {}", "MVM PANIC:".red().bold(), message);
}

pub fn info(start: impl AsRef<str>, message: impl std::fmt::Display) {
    println!("{} {}", start.as_ref().green().bold(), message);
}
