use std::{
    fs::File,
    io::{Read, Write},
};

use thrussh_keys::key::{ed25519, KeyPair, PublicKey};

pub mod serialize;

#[derive(Debug)]
pub struct SSHConfig {
    pub client_keys: Vec<PublicKey>,
    pub keypairs: Vec<KeyPair>,
}

impl SSHConfig {
    pub fn load(filename: &str) -> Self {
        let mut f = match File::open(&filename) {
            Ok(f) => f,
            Err(_) => return Self::default(),
        };
        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Ok(_bytes_read) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    println!("there was an error parsing that file: {e:?}");
                    Self::default()
                }
            },
            Err(e) => {
                println!("there was an error reading that file: {e:?}");
                Self::default()
            }
        }
    }

    pub fn save(&self, filename: &str) {
        let mut f = match File::create(&filename) {
            Ok(f) => f,
            Err(e) => {
                println!("There was an error creating that file: {e:?}");
                return;
            }
        };

        let contents = match toml::to_string(self) {
            Ok(contents) => contents,
            Err(e) => {
                println!("There was an error serializing that config: {e:?}");
                return;
            }
        };

        f.write_all(contents.as_bytes())
            .map_err(|e| println!("There was an error writing that file: {e:?}"))
            .ok();
    }
}

impl Default for SSHConfig {
    fn default() -> Self {
        Self {
            client_keys: vec![KeyPair::generate_ed25519().unwrap().clone_public_key()],
            keypairs: vec![KeyPair::generate_ed25519().unwrap()],
        }
    }
}

impl Clone for SSHConfig {
    fn clone(&self) -> Self {
        Self {
            client_keys: self
                .client_keys
                .iter()
                .map(|k| match k {
                    PublicKey::Ed25519(public) => PublicKey::Ed25519(ed25519::PublicKey {
                        key: public.key.clone(),
                    }),
                })
                .collect(),
            keypairs: self
                .keypairs
                .iter()
                .map(|k| match k {
                    KeyPair::Ed25519(secret) => KeyPair::Ed25519(ed25519::SecretKey {
                        key: secret.key.clone(),
                    }),
                })
                .collect(),
        }
    }
}
