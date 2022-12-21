use crate::cli::REPLArgs;
use futures::lock::Mutex;
use server::Server;
use std::{collections::HashMap, sync::Arc};

use self::config::SSHConfig;

pub mod config;
pub mod keys;
pub mod server;

const SSH_CONFIG_FILENAME: &str = "ssh.toml";

pub fn start_ssh_server(args: REPLArgs) {
    let _t = tokio::spawn(async move {
        let ssh_config = SSHConfig::load(SSH_CONFIG_FILENAME);
        ssh_config.save(SSH_CONFIG_FILENAME);

        let mut config = thrussh::server::Config::default();
        config.connection_timeout = Some(std::time::Duration::from_secs(600));
        config.auth_rejection_time = std::time::Duration::from_secs(3);
        config.keys.append(&mut ssh_config.clone().keypairs);

        let config = Arc::new(config);
        let sh = Server {
            client_pubkey: Arc::new(ssh_config.clone().client_keys.swap_remove(0)),
            clients: Arc::new(Mutex::new(HashMap::new())),
            id: 0,
            repl_mode: args.mode,
        };

        let addr: &str = &format!("0.0.0.0:{}", args.ssh_port);
        let res = thrussh::server::run(config, addr, sh).await;

        match res {
            Ok(v) => println!("{:?}", v),
            Err(err) => println!("{}", err),
        }

        ssh_config.save(SSH_CONFIG_FILENAME);
    });
}
