use clap::{command, Arg, Command};

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
                .index(1),
        )
}
