use clap::{command, Arg, ArgAction, Command};

pub fn cli() -> Command {
    command!()
        .name("rocky")
        .version("0.0.1")
        .author("Galitan-dev <galitan.dev@gmail.com>")
        .about("Interpreter for the Rocky language")
        .arg(
            Arg::new("input_file")
                .help("Path to the .rk file to run")
                .required(false)
                .index(1)
                .value_name("INPUT_FILE"),
        )
        .arg(
            Arg::new("threads")
                .help("Number of OS threads the VM will utilize")
                .required(false)
                .long("threads")
                .short('t')
                .value_name("threads"),
        )
        .arg(
            Arg::new("hexadecimal")
                .help("Use the REPL in hexadecimal (for you, little weirdo)")
                .required(false)
                .long("hexadecimal")
                .alias("hexa")
                .short('H')
                .action(ArgAction::SetTrue),
        )
}
