use clap::{arg, Command};

pub fn cli() -> Command {
    Command::new("mvm")
        .about("Mealet's 64-bit virtual machine")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .about("run virtual machine with compiled program")
                .arg(
                    arg!(--mem <MEMSIZE> "machine memory size in bytes")
                        .default_value("1024")
                        .required(false)
                )
                .arg(
                    arg!(--stack <STACKSIZE> "stack size in bytes")
                        .default_value("128")
                        .required(false)
                )
                .arg(
                    arg!(--log <FILE> "log machine to specified file")
                        .required(false)
                )
                .arg(arg!(<PROGRAM> "path to program binary file"))
        )
        .subcommand(
            Command::new("compile")
                .arg(
                    arg!(<ASM> "assembly file path")
                )
                .arg_required_else_help(true)
        )
}
