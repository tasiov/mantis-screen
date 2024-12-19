use crate::error::Error;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub fn pubkey_from_str(s: &str) -> Result<Pubkey, Error> {
    Pubkey::from_str(s).map_err(|e| Error::RpcClient(e.to_string()))
}
