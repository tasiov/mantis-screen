use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_token::ID as TOKEN_PROGRAM_ID;
use tracing::debug;

use crate::error::Error;
use crate::{commands::fetch_pool_keys::PoolKeys as RpcPoolKeys, utils::pubkey::pubkey_from_str};

#[derive(Debug)]
pub struct LiquidityAddInstructionParams<'a> {
    pub rpc_pool_keys: &'a RpcPoolKeys,
    pub user_keys: &'a IxUserKeys,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub fixed_side: FixedSide,
    pub other_amount_min: u64,
}

#[derive(Debug)]
pub struct IxVaultKeys {
    pub a: Pubkey,
    pub b: Pubkey,
}

#[derive(Debug)]
pub struct IxUserKeys {
    pub base_token_account: Pubkey,
    pub quote_token_account: Pubkey,
    pub lp_token_account: Pubkey,
    pub owner: Pubkey,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FixedSide {
    Base,
    Quote,
}

pub fn make_add_liquidity_instruction(
    params: LiquidityAddInstructionParams,
) -> Result<Instruction, Error> {
    debug!("Params: {:?}", params);

    // Create instruction data
    let mut data = Vec::with_capacity(33); // 1 + (4 * 8)
    data.push(3u8); // instruction discriminator
    data.extend_from_slice(&params.base_amount_in.to_le_bytes());
    data.extend_from_slice(&params.quote_amount_in.to_le_bytes());
    data.extend_from_slice(&params.other_amount_min.to_le_bytes());
    data.extend_from_slice(&match params.fixed_side {
        FixedSide::Base => 0u64.to_le_bytes(),
        FixedSide::Quote => 1u64.to_le_bytes(),
    });

    // Create account metas
    let mut keys = vec![
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new(pubkey_from_str(&params.rpc_pool_keys.id)?, false),
        AccountMeta::new_readonly(pubkey_from_str(&params.rpc_pool_keys.authority)?, false),
        AccountMeta::new_readonly(pubkey_from_str(&params.rpc_pool_keys.openOrders)?, false),
        AccountMeta::new(pubkey_from_str(&params.rpc_pool_keys.targetOrders)?, false),
        AccountMeta::new(
            pubkey_from_str(&params.rpc_pool_keys.mintLp.address)?,
            false,
        ),
        AccountMeta::new(pubkey_from_str(&params.rpc_pool_keys.vault.A)?, false),
        AccountMeta::new(pubkey_from_str(&params.rpc_pool_keys.vault.B)?, false),
    ];

    // Add remaining accounts
    keys.extend_from_slice(&[
        AccountMeta::new_readonly(pubkey_from_str(&params.rpc_pool_keys.marketId)?, false),
        AccountMeta::new(params.user_keys.base_token_account, false),
        AccountMeta::new(params.user_keys.quote_token_account, false),
        AccountMeta::new(params.user_keys.lp_token_account, false),
        AccountMeta::new_readonly(params.user_keys.owner, true),
        AccountMeta::new_readonly(
            pubkey_from_str(&params.rpc_pool_keys.marketEventQueue)?,
            false,
        ),
    ]);

    Ok(Instruction {
        program_id: pubkey_from_str(&params.rpc_pool_keys.programId)?,
        accounts: keys,
        data,
    })
}
