use solana_client::{rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::Hash,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    signer::EncodableKey,
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_token::{
    instruction::{close_account, initialize_account},
    solana_program::program_pack::Pack,
    state::Account as TokenAccount,
    ID as TOKEN_PROGRAM_ID,
};
use tracing::debug;

use crate::{config::Config, error::Error};

pub const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[derive(Debug)]
pub struct TokenAccountInfo {
    pub token_account: Pubkey,
    pub start_instructions: Vec<Instruction>,
    pub end_instructions: Vec<Instruction>,
    pub instruction_types: Vec<String>,
    pub additional_signers: Vec<Keypair>,
}

pub enum TokenSide {
    In,
    Out,
}

pub struct HandleTokenAccountParams {
    pub side: TokenSide,
    pub amount: u64,
    pub mint: Pubkey,
    pub token_account: Option<Pubkey>,
    pub bypass_associated_check: bool,
    pub skip_close_account: bool,
    pub check_create_ata_owner: bool,
}

pub struct Client {
    pub rpc_client: RpcClient,
    pub payer: Keypair,
}

pub fn get_client(config: &Config) -> Result<Client, Error> {
    let payer = Keypair::read_from_file(config.keypair_path.clone())
        .map_err(|e| Error::Keypair(e.to_string()))?;

    let rpc_endpoint = format!("{}?api-key={}", config.rpc_endpoint, config.api_key);

    let rpc_client = RpcClient::new_with_commitment(rpc_endpoint, CommitmentConfig::confirmed());

    Ok(Client { rpc_client, payer })
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

    pub fn get_token_account_balance_ui_amount(&self, address: &Pubkey) -> Result<f64, Error> {
        let ui_token_amount = self
            .rpc_client
            .get_token_account_balance(address)
            .map_err(|e| Error::RpcClient(e.to_string()))?;

        Ok(ui_token_amount.ui_amount.unwrap_or(0.0))
    }

    pub fn get_token_account_balance_string(&self, address: &Pubkey) -> Result<String, Error> {
        let ui_token_amount = self
            .rpc_client
            .get_token_account_balance(address)
            .map_err(|e| Error::RpcClient(e.to_string()))?;

        Ok(ui_token_amount.amount)
    }

    pub async fn handle_token_account(
        &self,
        params: HandleTokenAccountParams,
    ) -> Result<TokenAccountInfo, Error> {
        let HandleTokenAccountParams {
            side,
            amount,
            mint,
            token_account,
            bypass_associated_check,
            skip_close_account,
            check_create_ata_owner,
        } = params;

        // Get ATA for this mint
        let ata = get_associated_token_address_with_program_id(
            &self.get_payer_pubkey(),
            &mint,
            &TOKEN_PROGRAM_ID,
        );

        // Handle WSOL case
        if mint.to_string() == WSOL_MINT {
            let wsol_keypair = Keypair::new();
            let wsol_account = wsol_keypair.pubkey();

            // Get rent-exempt amount
            let min_balance = self
                .rpc_client
                .get_minimum_balance_for_rent_exemption(TokenAccount::LEN as usize)
                .map_err(|e| Error::RpcClient(e.to_string()))?;
            let transaction_fee = 5000; // Default fee
            let total_needed = amount + min_balance + transaction_fee;

            // Check SOL balance
            let sol_balance = self
                .rpc_client
                .get_balance(&self.get_payer_pubkey())
                .map_err(|e| Error::RpcClient(e.to_string()))?;

            if sol_balance < total_needed {
                return Err(Error::InsufficientBalance(format!(
                    "Insufficient SOL for wrap. Need {}, have {}",
                    total_needed, sol_balance
                )));
            }

            // Create instructions for WSOL handling
            let mut start_instructions = vec![];
            let mut end_instructions = vec![];

            // Create account
            start_instructions.push(create_account(
                &self.get_payer_pubkey(),
                &wsol_account,
                min_balance + amount,
                TokenAccount::LEN as u64,
                &spl_token::id(),
            ));

            // Initialize token account
            start_instructions.push(
                initialize_account(
                    &spl_token::id(),
                    &wsol_account,
                    &mint,
                    &self.get_payer_pubkey(),
                )
                .map_err(|e| Error::RpcClient(e.to_string()))?,
            );

            // Add close instruction if not skipped
            if !skip_close_account {
                end_instructions.push(
                    close_account(
                        &spl_token::id(),
                        &wsol_account,
                        &self.get_payer_pubkey(),
                        &self.get_payer_pubkey(),
                        &[],
                    )
                    .map_err(|e| Error::RpcClient(e.to_string()))?,
                );
            }

            return Ok(TokenAccountInfo {
                token_account: wsol_account,
                start_instructions,
                end_instructions,
                instruction_types: vec![
                    "CreateWSolAccount".to_string(),
                    "CloseWSolAccount".to_string(),
                ],
                additional_signers: vec![wsol_keypair],
            });
        }

        // Handle regular token or create ATA
        if token_account.is_none()
            || (matches!(side, TokenSide::Out)
                && token_account != Some(ata)
                && !bypass_associated_check)
        {
            let mut instructions = vec![];

            let create_ata_ix = create_associated_token_account(
                &self.get_payer_pubkey(),
                &self.get_payer_pubkey(),
                &mint,
                &TOKEN_PROGRAM_ID,
            );

            if check_create_ata_owner {
                // Check if ATA exists and is valid
                if let Ok(account) = self.rpc_client.get_account(&ata) {
                    if let Ok(token_account) = TokenAccount::unpack(&account.data) {
                        if token_account.mint == mint
                            && token_account.owner == self.get_payer_pubkey()
                        {
                            return Ok(TokenAccountInfo {
                                token_account: ata,
                                start_instructions: vec![],
                                end_instructions: vec![],
                                instruction_types: vec![],
                                additional_signers: vec![],
                            });
                        }
                    }
                    return Err(Error::InvalidTokenAccount(format!(
                        "create ata check error -> mint: {}, ata: {}",
                        mint, ata
                    )));
                }
            }

            instructions.push(create_ata_ix);

            return Ok(TokenAccountInfo {
                token_account: ata,
                start_instructions: instructions,
                end_instructions: vec![],
                instruction_types: vec!["CreateATA".to_string()],
                additional_signers: vec![],
            });
        }

        // Return existing token account
        Ok(TokenAccountInfo {
            token_account: token_account.unwrap_or(ata),
            start_instructions: vec![],
            end_instructions: vec![],
            instruction_types: vec![],
            additional_signers: vec![],
        })
    }

    pub fn send_transaction(
        &self,
        instructions: &[Instruction],
        additional_signers: &[Keypair],
    ) -> Result<Signature, Error> {
        debug!("Getting latest blockhash...");
        let recent_blockhash = self.get_latest_blockhash()?;
        debug!("Got blockhash: {}", recent_blockhash);

        let mut signers = vec![&self.payer];
        signers.extend(additional_signers);

        debug!(
            "Creating transaction with {} instructions...",
            instructions.len()
        );
        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.get_payer_pubkey()),
            &signers,
            recent_blockhash,
        );
        debug!("Transaction created with {} signers", signers.len());

        // Print info about each instruction
        for (i, instruction) in instructions.iter().enumerate() {
            debug!(
                "Instruction {}: Program {} with {} accounts",
                i,
                instruction.program_id,
                instruction.accounts.len()
            );
        }

        debug!("Sending transaction...");
        match self
            .rpc_client
            .send_and_confirm_transaction_with_spinner_and_config(
                &transaction,
                CommitmentConfig::confirmed(),
                RpcSendTransactionConfig {
                    skip_preflight: false,
                    preflight_commitment: Some(CommitmentConfig::processed().commitment),
                    encoding: None,
                    max_retries: Some(3),
                    min_context_slot: None,
                },
            ) {
            Ok(sig) => {
                debug!("Transaction confirmed with signature: {}", sig);
                Ok(sig)
            }
            Err(e) => {
                debug!("Transaction failed: {}", e);
                // Try to get more information about the failure
                if let Ok(simulation) = self.rpc_client.simulate_transaction(&transaction) {
                    debug!("Simulation logs: {:?}", simulation.value.logs);
                }
                Err(Error::RpcClient(e.to_string()))
            }
        }
    }
}
