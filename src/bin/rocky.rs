extern crate rocky;

use clap::{parser::RawValues, ArgMatches};
use rocky::{
    cli::{cli, AddSshKeyArgs, Args, REPLArgs, RunFileArgs},
    repl::REPLMode,
    run_file,
    ssh::start_ssh_server,
    start_repl,
};
use rustyline::error::ReadlineError;

#[tokio::main]
async fn main() -> Result<(), ReadlineError> {
    let matches = cli().get_matches();
    let args = get_arguments(&matches);

    match args {
        Args::RunFile(args) => run_file(args),
        Args::Repl(args) => {
            if args.enable_ssh {
                println!("Enabled SSH at port {}", args.ssh_port);
                start_ssh_server(args.clone());
            }
            start_repl(args)?
        }
        Args::AddSshKey(args) => {
            println!("User tried to add SSH key at {}!", args.pub_key_file)
        }
    }

    Ok(())
}

fn get_arguments<'a>(matches: &'a ArgMatches) -> Args<'a> {
    let (command, args) = matches.subcommand().map_or(("rocky", matches), |s| s);
    match command {
        "rocky" => match unwrap(args.get_raw("input_file")) {
            Some(input_file) => Args::RunFile(RunFileArgs {
                num_threads: match unwrap(args.get_raw("threads")) {
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
                filename: input_file,
                debug: args.get_flag("debug"),
            }),
            None => Args::Repl(REPLArgs {
                mode: {
                    if args.get_flag("hexadecimal") {
                        REPLMode::Hexadecimal
                    } else {
                        REPLMode::Assembly
                    }
                },
                enable_ssh: args.get_flag("enable_ssh"),
                ssh_port: unwrap(args.get_raw("ssh_port"))
                    .unwrap()
                    .parse()
                    .unwrap_or_else(|_| panic!("Invalid Port")),
            }),
        },
        "add-ssh-key" => Args::AddSshKey(AddSshKeyArgs {
            pub_key_file: unwrap(args.get_raw("pub_key_file")).unwrap(),
        }),
        _ => panic!("Invalid Command \"{command}\""),
    }
}

fn unwrap(raw: Option<RawValues>) -> Option<&str> {
    raw.map(|mut v| v.next().unwrap().to_str().unwrap())
}
