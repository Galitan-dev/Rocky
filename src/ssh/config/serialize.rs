use serde::{Deserialize, Serialize};
use thrussh_keys::key::{ed25519, KeyPair, PublicKey};

use super::SSHConfig;

#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize)]
struct SimpleConfig {
    client_keys: Vec<Vec<u8>>,
    server_keys: Vec<Vec<u8>>,
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
