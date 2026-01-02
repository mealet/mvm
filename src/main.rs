use colored::Colorize;

mod assembly;
mod cli;
mod vm;

pub const MEMSIZE_DEFAULT: usize = 1024;
pub const STACKSIZE_DEFAULT: usize = 256;

fn main() {
    let cli = cli::cli().try_get_matches().unwrap_or_else(|e| {
        let version = env!("CARGO_PKG_VERSION");
        let authors_env = env!("CARGO_PKG_AUTHORS");
        let authors_fmt = if authors_env.contains(":") {
            format!("\n| {}", authors_env.replace(":", "\n| "))
        } else {
            authors_env.to_owned()
        };

        match e.kind() {
            clap::error::ErrorKind::DisplayVersion => {
                eprintln!("{}", "⚡ Mealet Virtual Machine".bold().red());
                eprintln!("| - version: {version}");
                eprintln!("| - authors: {authors_fmt}");

                std::process::exit(0);
            }

            _ => {
                let bin = env!("CARGO_PKG_NAME");

                eprintln!("{}", "⚡ Mealet Virtual Machine".bold().red());
                eprintln!("| - version: {version}");
                eprintln!("| - authors: {authors_fmt}");
                eprintln!();
                eprintln!("{}", "🍀 Options:".bold().red());

                let _ = cli::cli().print_help();

                eprintln!();
                eprintln!("{}", "🎓 Examples of usage:".bold().red());
                eprintln!("  {bin} compile hello_world.asm");
                eprintln!("  {bin} run hello_world.mvm");
                eprintln!("  {bin} run hello_world.mvm -m 1024 -s 256");

                if e.kind() == clap::error::ErrorKind::DisplayHelp {
                    std::process::exit(0);
                }
                std::process::exit(1);
            }
        }
    });

    match cli.subcommand() {
        Some(("compile", sub_matches)) => {
            cli::info("Reading", "source code");

            let path_to_asm = sub_matches
                .get_one::<String>("ASM")
                .expect("asm path required");
            let code = std::fs::read_to_string(path_to_asm).unwrap_or_else(|err| {
                cli::error(format!("Unable to read assembly source code [{}]", err));
                std::process::exit(1);
            });

            let reporter = miette::GraphicalReportHandler::new();

            cli::info("Compiling", format!("assembly file ({})", path_to_asm));

            let mut lexer = assembly::lexer::Lexer::new("TEST", &code);
            let tokens = lexer.tokenize().unwrap_or_else(|errors| {
                for err in errors {
                    let mut buffer = String::new();
                    let _ = reporter.render_report(&mut buffer, err);

                    eprintln!("{}", buffer);
                }

                std::process::exit(1);
            });

            let mut parser = assembly::parser::Parser::new("TEST", &code, &tokens);
            let ast = parser.parse().unwrap_or_else(|errors| {
                for err in errors {
                    let mut buffer = String::new();
                    let _ = reporter.render_report(&mut buffer, err);

                    eprintln!("{}", buffer);
                }

                std::process::exit(1);
            });

            let mut analyzer = assembly::semantic::Analyzer::new("TEST", &code);
            analyzer.analyze(&ast).unwrap_or_else(|errors| {
                for err in errors {
                    let mut buffer = String::new();
                    let _ = reporter.render_report(&mut buffer, err);

                    eprintln!("{}", buffer);
                }

                std::process::exit(1);
            });

            // release mode flag
            let release_mode = sub_matches.get_flag("release");

            let mut codegen = assembly::codegen::Codegen::new(release_mode);
            let code = codegen.compile(&ast);

            cli::info("Writing", "generated code to binary");

            let new_file = path_to_asm.replace(".asm", ".mvm");

            std::fs::write(&new_file, code).unwrap_or_else(|err| {
                cli::error(format!(
                    "Unable to write generated code to binary file [{}]",
                    err
                ));
            });

            cli::info(
                "Successfully",
                format!("compiled assembly to mvm binary: {} ({} mode)", new_file, if release_mode { "release" } else { "debug" }),
            );
        }

        Some(("run", sub_matches)) => {
            const METADATA_MINIMUM_LENGTH: usize = 8 + 8;

            let memsize = sub_matches
                .get_one::<String>("MEMSIZE")
                .cloned()
                .unwrap_or(String::default());

            let stacksize = sub_matches
                .get_one::<String>("STACKSIZE")
                .cloned()
                .unwrap_or(String::default());

            let program_path = sub_matches
                .get_one::<String>("PROGRAM")
                .expect("no program path found");
            let mut program = std::fs::read(program_path).unwrap_or_else(|err| {
                cli::error(format!("Unable to read binary program [{}]", err));
                std::process::exit(1);
            });

            let mut memsize = memsize.parse::<usize>().unwrap_or(0);
            let mut stacksize = stacksize.parse::<usize>().unwrap_or(0);

            let mut metadata = Vec::new();

            let mut lptr = 0;
            let mut rptr = 1;

            while let Some(lhs) = program.get(lptr)
                && let Some(rhs) = program.get(rptr)
                && (*lhs != 0xFF || *rhs != vm::Opcode::DataSection as u8)
            {
                metadata.push(*lhs);

                lptr += 1;
                rptr += 1;
            }

            metadata.push(255);

            if !metadata.is_empty() && memsize != 0 && stacksize != 0 {
                if metadata.len() < METADATA_MINIMUM_LENGTH {
                    cli::error("Metadata's data is broken, please verify the file");
                    std::process::exit(1);
                }

                memsize = u64::from_be_bytes([
                    metadata[0],
                    metadata[1],
                    metadata[2],
                    metadata[3],
                    metadata[4],
                    metadata[5],
                    metadata[6],
                    metadata[7],
                ]) as usize;

                stacksize = u64::from_be_bytes([
                    metadata[8],
                    metadata[8 + 1],
                    metadata[8 + 2],
                    metadata[8 + 3],
                    metadata[8 + 4],
                    metadata[8 + 5],
                    metadata[8 + 6],
                    metadata[8 + 7],
                ]) as usize;
            }

            memsize = if memsize == 0 {
                MEMSIZE_DEFAULT
            } else {
                memsize
            };
            stacksize = if stacksize == 0 {
                STACKSIZE_DEFAULT
            } else {
                stacksize
            };

            program = program[metadata.len()..].to_vec();

            let mut vm = vm::VM::new(memsize, stacksize).unwrap_or_else(|err| {
                cli::error(format!("Unable to create VM instance [{}]", err));
                std::process::exit(1);
            });

            vm.insert_program(&program).unwrap_or_else(|err| {
                cli::error(format!("Unable to load the program [{}]", err));
            });

            vm.run().unwrap_or_else(|err| {
                cli::vm_error(err);
                std::process::exit(1);
            });

            std::process::exit(vm.exit_code as i32);
        }

        _ => unreachable!(),
    }
}
