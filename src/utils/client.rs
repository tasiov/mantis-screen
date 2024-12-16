use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::Hash,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    signer::EncodableKey,
    transaction::Transaction,
};
use spl_token::ID as TOKEN_PROGRAM_ID;

use crate::{config::Config, error::Error};

pub fn get_client(config: &Config) -> Result<Client, Error> {
    let payer = Keypair::read_from_file(config.keypair_path.clone())
        .map_err(|e| Error::Keypair(e.to_string()))?;

    let rpc_client = RpcClient::new_with_commitment(
        config.rpc_endpoint.to_string(),
        CommitmentConfig::confirmed(),
    );

    Ok(Client { rpc_client, payer })
}

pub struct Client {
    pub rpc_client: RpcClient,
    pub payer: Keypair,
}

impl Client {
    pub fn get_latest_blockhash(&self) -> Result<Hash, Error> {
        let (blockhash, _) = self
            .rpc_client
            .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
            .map_err(|e| Error::RpcClient(e.to_string()))?;

        Ok(blockhash)
    }

    pub fn get_payer_pubkey(&self) -> Pubkey {
        self.payer.pubkey()
    }

    pub fn get_or_create_token_account(
        &self,
        mint: &Pubkey,
    ) -> Result<(Pubkey, Option<Instruction>), Error> {
        let owner = &self.payer.pubkey();

        // Get associated token account address
        let ata = spl_associated_token_account::get_associated_token_address_with_program_id(
            owner,
            mint,
            &TOKEN_PROGRAM_ID,
        );

        // Check if account exists
        match self.rpc_client.get_account(&ata) {
            Ok(_) => {
                // Account exists
                Ok((ata, None))
            }
            Err(_) => {
                // Need to create account
                let create_ata_ix =
                    spl_associated_token_account::instruction::create_associated_token_account(
                        &owner,
                        &owner,
                        mint,
                        &TOKEN_PROGRAM_ID,
                    );

                Ok((ata, Some(create_ata_ix)))
            }
        }
    }

    pub fn send_transaction(&self, instructions: &[Instruction]) -> Result<Signature, Error> {
        let recent_blockhash = self.get_latest_blockhash()?;

        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.get_payer_pubkey()),
            &[&self.payer],
            recent_blockhash,
        );

        let signature = self
            .rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &transaction,
                CommitmentConfig::processed(),
            )
            .map_err(|e| Error::RpcClient(e.to_string()))?;

        Ok(signature)
    }
}
