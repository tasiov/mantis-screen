use crate::{
    commands::{
        fetch_pool_info::{fetch_pool_info, PoolInfo},
        fetch_pool_keys::{fetch_pool_keys, PoolKeys},
    },
    config::Config,
    error::Error,
    instructions::remove_liquidity::{
        make_remove_liquidity_instruction, IxUserKeys, RemoveLiquidityInstructionParams,
    },
    utils::{
        amount::{amount_display_to_raw, amount_raw_to_display},
        client::{Client, HandleTokenAccountParams, TokenSide},
        compute_budget::{add_compute_budget, ComputeBudgetConfig},
        confirmation::get_confirmation,
        pubkey::pubkey_from_str,
    },
};

use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
};
use std::str::FromStr;
use tracing::{debug, info};

pub async fn execute(
    config: &Config,
    client: &Client,
    pool_id: &str,
    lp_amount: f64,
    slippage_percentage: f64,
    base_amount_min: f64,
    quote_amount_min: f64,
) -> Result<(), Error> {
    let api_response = fetch_pool_info(config, pool_id).await?;
    let pool_info = api_response.data.first().unwrap();
    let pool_keys = fetch_pool_keys(config, pool_id).await?;
    let pool_keys = pool_keys.data.first().unwrap();

    let signature = remove_liquidity(
        client,
        pool_info,
        pool_keys,
        lp_amount,
        slippage_percentage,
        base_amount_min,
        quote_amount_min,
    )
    .await?;

    if let Some(signature) = signature {
        info!("Transaction sent with signature: {}", signature);
    } else {
        info!("No transaction sent");
    }

    Ok(())
}

async fn remove_liquidity(
    client: &Client,
    pool_info: &PoolInfo,
    pool_keys: &PoolKeys,
    lp_amount: f64,
    slippage_percentage: f64,
    base_amount_min: f64,
    quote_amount_min: f64,
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

    let mint_a = pubkey_from_str(&pool_keys.mintA.address)?;
    let mint_b = pubkey_from_str(&pool_keys.mintB.address)?;
    let mint_lp = pubkey_from_str(&pool_keys.mintLp.address)?;

    let lp_amount_raw = amount_display_to_raw(lp_amount, pool_info.lpMint.decimals)
        .to_string()
        .parse::<u64>()
        .unwrap();

    let base_amount_min_raw = amount_display_to_raw(base_amount_min, pool_info.mintA.decimals)
        .to_string()
        .parse::<u64>()
        .unwrap();

    let quote_amount_min_raw = amount_display_to_raw(quote_amount_min, pool_info.mintB.decimals)
        .to_string()
        .parse::<u64>()
        .unwrap();

    // Handle LP token account (input)
    let token_lp_info = client
        .handle_token_account(HandleTokenAccountParams {
            side: TokenSide::In,
            amount: lp_amount_raw,
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

    // Handle base token account (output)
    let token_a_info = client
        .handle_token_account(HandleTokenAccountParams {
            side: TokenSide::Out,
            amount: 0,
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

    // Handle quote token account (output)
    let token_b_info = client
        .handle_token_account(HandleTokenAccountParams {
            side: TokenSide::Out,
            amount: 0,
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

    // Check user balances
    match client.get_token_account_balance_ui_amount(&token_lp_info.token_account) {
        Ok(balance) => {
            info!("User LP token balance: {} {}", mint_lp, balance);
        }
        Err(e) => {
            info!("Failed getting user LP token balance: {}", e);
        }
    };

    let lp_amount_display = amount_raw_to_display(lp_amount_raw, pool_info.lpMint.decimals);
    let base_min_display = amount_raw_to_display(base_amount_min_raw, pool_info.mintA.decimals);
    let quote_min_display = amount_raw_to_display(quote_amount_min_raw, pool_info.mintB.decimals);

    let confirmation_msg = format!(
        "Remove Liquidity: {} LP tokens for minimum {} {} and {} {}",
        lp_amount_display,
        base_min_display,
        pool_info.mintA.symbol,
        quote_min_display,
        pool_info.mintB.symbol,
    );

    get_confirmation(&confirmation_msg);

    let remove_liquidity_ix =
        make_remove_liquidity_instruction(RemoveLiquidityInstructionParams {
            rpc_pool_keys: &pool_keys,
            user_keys: &IxUserKeys {
                lp_token_account: token_lp_info.token_account,
                base_token_account: token_a_info.token_account,
                quote_token_account: token_b_info.token_account,
                owner: client.get_payer_pubkey(),
            },
            lp_amount: lp_amount_raw,
            base_amount_min: base_amount_min_raw,
            quote_amount_min: quote_amount_min_raw,
        })?;

    instruction_options.push(Some(remove_liquidity_ix));

    for ix in &token_lp_info.end_instructions {
        instruction_options.push(Some(ix.clone()));
    }
    for ix in &token_a_info.end_instructions {
        instruction_options.push(Some(ix.clone()));
    }
    for ix in &token_b_info.end_instructions {
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
