use crate::{commands::fetch_pool_keys::PoolKeys, error::Error, utils::pubkey::pubkey_from_str};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_token::ID as TOKEN_PROGRAM_ID;

pub struct RemoveLiquidityInstructionParams<'a> {
    pub rpc_pool_keys: &'a PoolKeys,
    pub user_keys: &'a IxUserKeys,
    pub lp_amount: u64,
    pub base_amount_min: u64,
    pub quote_amount_min: u64,
}

#[derive(Debug)]
pub struct IxUserKeys {
    pub base_token_account: Pubkey,
    pub quote_token_account: Pubkey,
    pub lp_token_account: Pubkey,
    pub owner: Pubkey,
}

pub fn make_remove_liquidity_instruction(
    params: RemoveLiquidityInstructionParams,
) -> Result<Instruction, Error> {
    let RemoveLiquidityInstructionParams {
        rpc_pool_keys,
        user_keys,
        lp_amount,
        base_amount_min,
        quote_amount_min,
    } = params;

    // Create instruction data buffer
    let mut data = vec![4u8]; // instruction discriminator
    data.extend_from_slice(&lp_amount.to_le_bytes());
    data.extend_from_slice(&base_amount_min.to_le_bytes());
    data.extend_from_slice(&quote_amount_min.to_le_bytes());

    // Build account metas
    let mut keys = vec![
        // System
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        // AMM accounts
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.id)?, false),
        AccountMeta::new_readonly(pubkey_from_str(&rpc_pool_keys.authority)?, false),
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.openOrders)?, false),
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.targetOrders)?, false),
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.mintLp.address)?, false),
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.vault.A)?, false),
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.vault.B)?, false),
    ];

    // Add version-specific accounts
    let pool_id = pubkey_from_str(&rpc_pool_keys.id)?;
    keys.push(AccountMeta::new_readonly(pool_id, false));
    keys.push(AccountMeta::new_readonly(pool_id, false));

    // Add Serum market accounts
    keys.extend_from_slice(&[
        AccountMeta::new_readonly(pubkey_from_str(&rpc_pool_keys.marketProgramId)?, false),
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.marketId)?, false),
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.marketBaseVault)?, false),
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.marketQuoteVault)?, false),
        AccountMeta::new_readonly(pubkey_from_str(&rpc_pool_keys.marketAuthority)?, false),
    ]);

    // Add user accounts
    keys.extend_from_slice(&[
        AccountMeta::new(user_keys.lp_token_account, false),
        AccountMeta::new(user_keys.base_token_account, false),
        AccountMeta::new(user_keys.quote_token_account, false),
        AccountMeta::new_readonly(user_keys.owner, true),
    ]);

    // Add Serum orderbook accounts
    keys.extend_from_slice(&[
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.marketEventQueue)?, false),
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.marketBids)?, false),
        AccountMeta::new(pubkey_from_str(&rpc_pool_keys.marketAsks)?, false),
    ]);

    Ok(Instruction {
        program_id: pubkey_from_str(&rpc_pool_keys.programId)?,
        accounts: keys,
        data,
    })
}
