// TODO: Remove this when release
#![allow(unused)]

mod cli;
mod vm;
mod assembly;

fn main() {
    // TODO: Implement fancy CLI

    let cli = cli::cli().get_matches();

    match cli.subcommand() {
        Some(("compile", sub_matches)) => {
            let path_to_asm = sub_matches.get_one::<String>("ASM").expect("asm path required");
            let code = std::fs::read_to_string(path_to_asm).expect("unable to read");

            let reporter = miette::GraphicalReportHandler::new();

            let mut lexer = assembly::lexer::Lexer::new("TEST", code);
            let tokens = lexer.tokenize().unwrap_or_else(|errors| {
                for err in errors {
                    let mut buffer = String::new();
                    reporter.render_report(&mut buffer, err);

                    eprintln!("{}", buffer);
                }

                std::process::exit(1);
            });

            for token in tokens {
                println!("{:?}", token);
            }
        },

        Some(("run", sub_matches)) => {
            let memsize = sub_matches.get_one::<String>("MEMSIZE").expect("no memsize found");
            let stacksize = sub_matches.get_one::<String>("STACKSIZE").expect("no stacksize found");
            let log_mode = match sub_matches.get_one::<String>("LOG") {
                Some(str) => str == "true",
                _ => false
            };

            let program_path = sub_matches.get_one::<String>("PROGRAM").expect("no program path found");
            let program = std::fs::read(program_path).expect("unable to read program");

            let memsize = memsize.parse::<usize>().expect("unable to parse memsize");
            let stacksize = stacksize.parse::<usize>().expect("unable to parse memsize");

            let mut vm = vm::VM::new(memsize, stacksize).unwrap_or_else(|err| {
                eprintln!("{}", err);
                std::process::exit(1);
            });

            vm.insert_program(&program);
            vm.run();

            dbg!(vm.memory.inner);
        }

        _ => unreachable!()
    }
}
