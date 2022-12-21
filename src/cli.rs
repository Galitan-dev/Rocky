use clap::{command, Arg, ArgAction, Command};

pub fn cli() -> Command {
    command!()
        .name("rocky")
        .version("0.0.1")
        .author("Galitan-dev <galitan.dev@gmail.com>")
        .about("Interpreter for the Rocky language")
        .args([
            Arg::new("input_file")
                .help("Path to the .rk file to run")
                .required(false)
                .index(1)
                .value_name("INPUT_FILE"),
            Arg::new("threads")
                .help("Number of OS threads the VM will utilize")
                .required(false)
                .long("threads")
                .short('t')
                .value_name("threads"),
            Arg::new("hexadecimal")
                .help("Use the REPL in hexadecimal (for you, little weirdo)")
                .required(false)
                .long("hexadecimal")
                .alias("hexa")
                .short('H')
                .action(ArgAction::SetTrue),
            Arg::new("enable_ssh")
                .help("Enables the SSH server component of Rocky VM")
                .required(false)
                .long("enable-ssh")
                .alias("ssh")
                .short('s')
                .action(ArgAction::SetTrue),
            Arg::new("ssh_port")
                .help("Which port Iridium should listen for SSH connections on")
                .required(false)
                .long("ssh-port")
                .alias("port")
                .short('p')
                .action(ArgAction::SetTrue),
        ])
        .subcommand(
            command!()
                .name("add-ssh-key")
                .about(
                    "Adds a public key to the list of keys authorized to access this VM remotely",
                )
                .version("0.0.1")
                .author("Galitan-dev <galitan.dev@gmail.com>")
                .args([Arg::new("pub_key_file")
                    .help("Path to the file containing the public key")
                    .required(true)
                    .index(1)
                    .value_name("PUB_KEY_FILE")]),
        )
}
