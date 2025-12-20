use colored::Colorize;

mod cli;
mod vm;
mod assembly;

fn main() {
    // TODO: Add memory && stack size reader for MVM format

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
                eprintln!("");
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

            let path_to_asm = sub_matches.get_one::<String>("ASM").expect("asm path required");
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
            let _ = analyzer.analyze(&ast).unwrap_or_else(|errors| {
                for err in errors {
                    let mut buffer = String::new();
                    let _ = reporter.render_report(&mut buffer, err);

                    eprintln!("{}", buffer);
                }

                std::process::exit(1);
            });

            let mut codegen = assembly::codegen::Codegen::new();
            let code = codegen.compile(&ast);

            cli::info("Writing", "generated code to binary");

            let new_file = path_to_asm.replace(".asm", ".mvm");

            std::fs::write(&new_file, code).unwrap_or_else(|err| {
                cli::error(format!("Unable to write generated code to binary file [{}]", err));
            });

            cli::info("Successfully", format!("compiled assembly to mvm binary: {}", new_file));
        },

        Some(("run", sub_matches)) => {
            let memsize = sub_matches.get_one::<String>("MEMSIZE").expect("no memsize found");
            let stacksize = sub_matches.get_one::<String>("STACKSIZE").expect("no stacksize found");

            let program_path = sub_matches.get_one::<String>("PROGRAM").expect("no program path found");
            let program = std::fs::read(program_path).expect("unable to read program");

            let memsize = memsize.parse::<usize>().expect("unable to parse memsize");
            let stacksize = stacksize.parse::<usize>().expect("unable to parse memsize");

            let mut vm = vm::VM::new(memsize, stacksize).unwrap_or_else(|err| {
                cli::error(format!("Unable to create VM instance [{}]", err));
                std::process::exit(1);
            });

            vm.insert_program(&program).unwrap_or_else(|err| {
                cli::error(format!("Unable to load the program [{}]", err));
            });

            let _ = vm.run().unwrap_or_else(|err| {
                cli::vm_error(err);
                std::process::exit(1);
            });
        }

        _ => unreachable!()
    }
}
