use crate::{
    commands::{
        fetch_pool_info::{fetch_pool_info, PoolInfo},
        fetch_pool_keys::{fetch_pool_keys, PoolKeys},
    },
    config::Config,
    error::Error,
    instructions::add_liquidity::{
        make_add_liquidity_instruction, FixedSide, IxUserKeys, LiquidityAddInstructionParams,
    },
    utils::{
        amount::{amount_display_to_raw, amount_raw_to_display},
        client::{Client, HandleTokenAccountParams, TokenSide},
        compute_budget::{add_compute_budget, ComputeBudgetConfig},
        confirmation::get_confirmation,
        pubkey::pubkey_from_str,
    },
};

use rust_decimal::{prelude::FromPrimitive, Decimal};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
};
use std::ops::Sub;
use std::str::FromStr;
use tracing::{debug, info};

pub async fn execute(
    config: &Config,
    client: &Client,
    pool_id: &str,
    base_mint_pubkey: &str,
    base_amount: f64,
    slippage_percentage: f64,
) -> Result<(), Error> {
    let api_response = fetch_pool_info(config, pool_id).await?;
    let pool_info = api_response.data.first().unwrap();
    let pool_keys = fetch_pool_keys(config, pool_id).await?;
    let pool_keys = pool_keys.data.first().unwrap();

    let base_mint_pubkey = pubkey_from_str(base_mint_pubkey)?;
    let signature = add_liquidity(
        client,
        pool_info,
        pool_keys,
        &base_mint_pubkey,
        base_amount,
        slippage_percentage,
    )
    .await?;

    if let Some(signature) = signature {
        info!("Transaction sent with signature: {}", signature);
    } else {
        info!("No transaction sent");
    }

    Ok(())
}

async fn add_liquidity(
    client: &Client,
    pool_info: &PoolInfo,
    pool_keys: &PoolKeys,
    input_mint_pubkey: &Pubkey,
    input_amount: f64,
    slippage_percentage: f64,
) -> Result<Option<Signature>, Error> {
    let mut instruction_options: Vec<Option<Instruction>> = vec![];
    let mut additional_signers: Vec<Keypair> = vec![];

    let compute_budget_ixs = add_compute_budget(&ComputeBudgetConfig {
        micro_lamports: Some(1_000_000),
        units: Some(1_000_000),
    });
    for ix in compute_budget_ixs {
        instruction_options.push(Some(ix.instruction));
    }

    let mint_a =
        Pubkey::from_str(&pool_keys.mintA.address).map_err(|e| Error::RpcClient(e.to_string()))?;
    let mint_b =
        Pubkey::from_str(&pool_keys.mintB.address).map_err(|e| Error::RpcClient(e.to_string()))?;
    let mint_lp =
        Pubkey::from_str(&pool_keys.mintLp.address).map_err(|e| Error::RpcClient(e.to_string()))?;

    let (input_amount, other_amount, min_other_amount) = calculate_values_from_input(
        client,
        pool_info,
        pool_keys,
        input_mint_pubkey,
        input_amount,
        slippage_percentage,
    )?;

    let (fixed_side, base_amount, quote_amount) = if input_mint_pubkey == &mint_a {
        (FixedSide::Base, input_amount, other_amount)
    } else {
        (FixedSide::Quote, other_amount, input_amount)
    };

    let token_a_info = client
        .handle_token_account(HandleTokenAccountParams {
            side: TokenSide::In,
            amount: base_amount,
            mint: mint_a,
            token_account: None,
            bypass_associated_check: false,
            skip_close_account: false,
            check_create_ata_owner: true,
        })
        .await?;
    for ix in &token_a_info.start_instructions {
        instruction_options.push(Some(ix.clone()));
    }
    for signer in token_a_info.additional_signers {
        additional_signers.push(signer);
    }

    let token_b_info = client
        .handle_token_account(HandleTokenAccountParams {
            side: TokenSide::In,
            amount: quote_amount,
            mint: mint_b,
            token_account: None,
            bypass_associated_check: false,
            skip_close_account: false,
            check_create_ata_owner: true,
        })
        .await?;
    for ix in &token_b_info.start_instructions {
        instruction_options.push(Some(ix.clone()));
    }
    for signer in token_b_info.additional_signers {
        additional_signers.push(signer);
    }

    let token_lp_info = client
        .handle_token_account(HandleTokenAccountParams {
            side: TokenSide::Out, // Because we're receiving LP tokens
            amount: 0,            // Initial LP token amount is 0
            mint: mint_lp,
            token_account: None,
            bypass_associated_check: false,
            skip_close_account: false,
            check_create_ata_owner: true,
        })
        .await?;
    for ix in &token_lp_info.start_instructions {
        instruction_options.push(Some(ix.clone()));
    }
    for signer in token_lp_info.additional_signers {
        additional_signers.push(signer);
    }

    // Check the user balances of the input mint and the other mint
    match client.get_token_account_balance_ui_amount(&token_a_info.token_account) {
        Ok(balance) => {
            info!("User balance mint a: {} {}", mint_a, balance);
        }
        Err(e) => {
            info!("Failed getting user balance mint a: {}", e);
        }
    };

    match client.get_token_account_balance_ui_amount(&token_b_info.token_account) {
        Ok(balance) => {
            info!("User balance mint b: {} {}", mint_b, balance);
        }
        Err(e) => {
            info!("Failed getting user balance mint b: {}", e);
        }
    };

    let base_amount_display = amount_raw_to_display(base_amount, pool_info.mintA.decimals);
    let quote_amount_display = amount_raw_to_display(quote_amount, pool_info.mintB.decimals);
    let min_other_amount_display =
        amount_raw_to_display(min_other_amount, pool_info.mintB.decimals);

    let confirmation_msg = format!(
        "{} Amount: {}, {} Amount: {}, Min Other Amount: {}",
        pool_info.mintA.symbol,
        base_amount_display,
        pool_info.mintB.symbol,
        quote_amount_display,
        min_other_amount_display
    );

    get_confirmation(&confirmation_msg);

    let add_liquidity_ix = make_add_liquidity_instruction(LiquidityAddInstructionParams {
        rpc_pool_keys: &pool_keys,
        user_keys: &IxUserKeys {
            base_token_account: token_a_info.token_account,
            quote_token_account: token_b_info.token_account,
            lp_token_account: token_lp_info.token_account,
            owner: client.get_payer_pubkey(),
        },
        base_amount_in: base_amount,
        quote_amount_in: quote_amount,
        fixed_side,
        other_amount_min: min_other_amount,
    })?;

    instruction_options.push(Some(add_liquidity_ix));

    for ix in &token_a_info.end_instructions {
        instruction_options.push(Some(ix.clone()));
    }
    for ix in &token_b_info.end_instructions {
        instruction_options.push(Some(ix.clone()));
    }
    for ix in &token_lp_info.end_instructions {
        instruction_options.push(Some(ix.clone()));
    }

    let instructions = instruction_options
        .into_iter()
        .filter(|ix| ix.is_some())
        .map(|ix| ix.unwrap())
        .collect::<Vec<Instruction>>();

    let mut signature = None;

    if !instructions.is_empty() {
        signature = Some(client.send_transaction(&instructions, &additional_signers)?);
    }

    Ok(signature)
}

fn calculate_values_from_input(
    client: &Client,
    pool_info: &PoolInfo,
    pool_keys: &PoolKeys,
    input_mint_pubkey: &Pubkey,
    input_amount: f64,
    slippage_percentage: f64,
) -> Result<(u64, u64, u64), Error> {
    let (input_mint, _) = if &input_mint_pubkey.to_string() == &pool_keys.mintA.address {
        (&pool_keys.mintA, &pool_keys.mintB)
    } else {
        (&pool_keys.mintB, &pool_keys.mintA)
    };

    // Convert input amount to u128
    let input_amount_raw = amount_display_to_raw(input_amount, input_mint.decimals)
        .to_string()
        .parse::<u128>()
        .unwrap();
    debug!("Input Amount Raw: {}", input_amount_raw);

    // Fetch the reserves from the pool token accounts
    let (input_reserve, other_reserve) =
        if &input_mint_pubkey.to_string() == &pool_info.mintA.address {
            (
                client
                    .get_token_account_balance_string(&pubkey_from_str(&pool_keys.vault.A)?)?
                    .parse::<u128>()
                    .unwrap(),
                client
                    .get_token_account_balance_string(&pubkey_from_str(&pool_keys.vault.B)?)?
                    .parse::<u128>()
                    .unwrap(),
            )
        } else {
            (
                client
                    .get_token_account_balance_string(&pubkey_from_str(&pool_keys.vault.B)?)?
                    .parse::<u128>()
                    .unwrap(),
                client
                    .get_token_account_balance_string(&pubkey_from_str(&pool_keys.vault.A)?)?
                    .parse::<u128>()
                    .unwrap(),
            )
        };

    debug!("Input Reserve: {}", input_reserve);
    debug!("Other Reserve: {}", other_reserve);

    let next_input_reserve = input_reserve + input_amount_raw;
    debug!("Next Input Reserve: {}", next_input_reserve);

    // Calculate constant product k = x * y
    let k = input_reserve
        .checked_mul(other_reserve)
        .ok_or_else(|| Error::Math("Overflow in k calculation".to_string()))?;
    debug!("Constant k: {}", k);

    // Calculate new other reserve: k / next_input_reserve
    let next_other_reserve = k
        .checked_div(next_input_reserve)
        .ok_or_else(|| Error::Math("Division error".to_string()))?;
    debug!("Next Other Reserve: {}", next_other_reserve);

    // Calculate other amount
    let other_amount = other_reserve
        .checked_sub(next_other_reserve)
        .ok_or_else(|| Error::Math("Subtraction error".to_string()))?;
    debug!("Other Amount: {}", other_amount);

    let slippage_decimal = Decimal::from_f64(slippage_percentage).unwrap() / Decimal::from(100u64);
    debug!("Slippage Decimal: {}", slippage_decimal);
    let coefficient = Decimal::ONE
        .sub(slippage_decimal)
        .to_string()
        .parse::<f64>()
        .unwrap();

    // Calculate minimum amounts using the same process
    let min_other_amount = (other_amount as f64 * coefficient) as u128;
    debug!("Min Other Amount: {}", min_other_amount);

    // Convert back to u64 for return values
    Ok((
        input_amount_raw
            .try_into()
            .map_err(|_| Error::Math("Overflow converting to u64".to_string()))?,
        other_amount
            .try_into()
            .map_err(|_| Error::Math("Overflow converting to u64".to_string()))?,
        min_other_amount
            .try_into()
            .map_err(|_| Error::Math("Overflow converting to u64".to_string()))?,
    ))
}
