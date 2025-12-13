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

            println!("{:?}", tokens);
        },

        Some(("run", sub_matches)) => {
            todo!()
        }

        _ => unreachable!()
    }
}
