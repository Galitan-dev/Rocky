use std::{
    fs::File,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};
use thrussh_keys::key::{ed25519, KeyPair, PublicKey};

#[derive(Debug)]
pub struct SSHConfig {
    pub client_keys: Vec<PublicKey>,
    pub keypairs: Vec<KeyPair>,
}

#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize)]
struct SimpleConfig {
    client_keys: Vec<Vec<u8>>,
    server_keys: Vec<Vec<u8>>,
}

impl Default for SSHConfig {
    fn default() -> Self {
        Self {
            client_keys: vec![KeyPair::generate_ed25519().unwrap().clone_public_key()],
            keypairs: vec![KeyPair::generate_ed25519().unwrap()],
        }
    }
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

impl Serialize for SSHConfig {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut config = SimpleConfig {
            client_keys: Vec::new(),
            server_keys: Vec::new(),
        };

        for key in &self.client_keys {
            config.client_keys.push(match key {
                PublicKey::Ed25519(public) => public.key.to_vec(),
            })
        }

        for keypair in &self.keypairs {
            config.server_keys.push(match keypair {
                KeyPair::Ed25519(secret) => secret.key.to_vec(),
            });
        }

        config.serialize(s)
    }
}

impl<'de> Deserialize<'de> for SSHConfig {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let config: SimpleConfig = Deserialize::deserialize(d)?;
        let mut ssh_config = SSHConfig {
            client_keys: Vec::new(),
            keypairs: Vec::new(),
        };

        for key in config.client_keys {
            ssh_config
                .client_keys
                .push(thrussh_keys::key::PublicKey::Ed25519(ed25519::PublicKey {
                    key: vec_to_array(key),
                }))
        }

        for key in config.server_keys {
            ssh_config
                .keypairs
                .push(thrussh_keys::key::KeyPair::Ed25519(ed25519::SecretKey {
                    key: vec_to_array(key),
                }))
        }

        Result::<SSHConfig, D::Error>::Ok(ssh_config)
    }
}

fn vec_to_array<T, const N: usize>(vec: Vec<T>) -> [T; N]
where
    T: Default + Copy,
{
    let mut array = [T::default(); N];
    array.copy_from_slice(&vec);
    array
}
