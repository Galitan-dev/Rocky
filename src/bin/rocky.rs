extern crate rocky;

use clap::{parser::RawValues, ArgMatches};
use rocky::{cli::cli, repl::REPLMode, run_file, start_repl};
use rustyline::error::ReadlineError;

fn main() -> Result<(), ReadlineError> {
    let matches = cli().get_matches();
    let args = get_arguments(&matches);

    match args {
        Args::RunFile((num_threads, filename)) => run_file(num_threads, filename),
        Args::Repl(mode) => start_repl(mode)?,
        Args::AddSshKey(pub_key_file) => println!("User tried to add SSH key at {pub_key_file}!"),
    }

    Ok(())
}

fn get_arguments<'a>(matches: &'a ArgMatches) -> Args<'a> {
    let (command, args) = matches.subcommand().map_or(("rocky", matches), |s| s);
    match command {
        "rocky" => match unwrap(args.get_raw("input_file")) {
            Some(input_file) => Args::RunFile((
                match unwrap(args.get_raw("threads")) {
                    Some(number) => match number.parse::<usize>() {
                        Ok(v) => v,
                        Err(_e) => {
                            println!(
                                "Invalid argument for number of threads: {}. Using default.",
                                number
                            );
                            num_cpus::get()
                        }
                    },
                    None => num_cpus::get(),
                },
                input_file,
            )),
            None => Args::Repl({
                if args.get_flag("hexadecimal") {
                    REPLMode::Hexadecimal
                } else {
                    REPLMode::Assembly
                }
            }),
        },
        "add-ssh-key" => Args::AddSshKey(unwrap(args.get_raw("pub_key_file")).unwrap()),
        _ => panic!("Invalid Command \"{command}\""),
    }
}

fn unwrap(raw: Option<RawValues>) -> Option<&str> {
    raw.map(|mut v| v.next().unwrap().to_str().unwrap())
}

#[derive(Debug, Clone)]
enum Args<'a> {
    Repl(REPLMode),
    RunFile((usize, &'a str)),
    AddSshKey(&'a str),
}
