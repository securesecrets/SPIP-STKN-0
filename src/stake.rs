use std::collections::BinaryHeap;
use cosmwasm_std::{Api, Binary, CanonicalAddr, Env, Extern, from_binary, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary, Uint128};
use secret_toolkit::snip20::{register_receive_msg, send_msg};
use shade_protocol::shd_staking::ReceiveType;
use shade_protocol::shd_staking::stake::{DailyUnbonding, StakeConfig, Unbonding};
use shade_protocol::storage::{BucketStorage, SingletonStorage};
use shade_protocol::utils::asset::Contract;
use crate::contract::{check_if_admin, try_mint_impl};
use crate::msg::HandleAnswer;
use crate::msg::ResponseStatus::Success;
use crate::state_staking::{DailyUnbondingQueue, TotalShares, TotalTokens, UnbondingQueue, UnsentStakedTokens, UserShares};
use crate::state::{Balances, Config, Constants};
use crate::transaction_history::{store_add_reward, store_stake, store_unbond};

pub fn try_update_stake_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    unbond_time: Option<u64>,
    disable_treasury: bool,
    treasury: Option<HumanAddr>,
) -> StdResult<HandleResponse> {
    let mut config = Config::from_storage(&mut deps.storage);

    check_if_admin(&config, &env.message.sender)?;

    let mut stake_config = StakeConfig::load(&deps.storage)?;

    if let Some(unbond_time) = unbond_time {
        stake_config.unbond_time = unbond_time;
    }

    let mut message = vec![];

    if disable_treasury {
        stake_config.treasury = None;
    }
    else if let Some(treasury) = treasury {
        stake_config.treasury = Some(treasury.clone());

        let unsent_tokens = UnsentStakedTokens::load(&deps.storage)?;
        if unsent_tokens.0 != 0 {
            message.push(send_msg(
                treasury,
                Uint128(unsent_tokens.0),
                None,
                None,
                None,
                u128,
                stake_config.staked_token.code_hash,
                stake_config.staked_token.address
            )?);
            UnsentStakedTokens(0).save(&mut deps.storage)?;
        }
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateStakeConfig { status: Success })?),
    })
}

///
/// Rounds down a date to the nearest day
///
fn round_date(date: &u64) -> u64 {
    let day = 86400; //60 * 60 * 24
    date - (date % day)
}

///
/// Updates total states to reflect balance changes
///
fn add_balance<S: Storage>(
    storage: &mut S,
    config: &mut Config<S>,
    stake_config: &StakeConfig,
    sender: &HumanAddr,
    sender_canon: &CanonicalAddr,
    amount: u128
) -> StdResult<()> {
    // Check if user account exists
    let mut user_shares = UserShares::may_load(
        &deps.storage,
        sender.as_str().as_bytes()
    )?.unwrap_or(UserShares(0));

    // Get total supplied tokens
    let mut total_shares = TotalShares::load(&deps.storage)?;
    let mut total_tokens = TotalTokens::load(&deps.storage)?;

    // Calculate shares per token supplied
    let shares = shares_per_token(
        &stake_config,
        &amount,
        &total_tokens.0,
        &total_shares.0,
    );

    // Update user's shares
    user_shares.0 += shares;
    user_shares.save(&mut deps.storage, sender.as_str().as_bytes())?;

    // Update total shares
    total_shares.0 += shares;
    total_shares.save(&mut deps.storage)?;

    // Update total staked
    config.set_total_supply(config.total_supply() + amount);
    total_tokens.0 += amount;
    total_tokens.save(&mut deps.storage)?;

    // Update user staked / tokens
    let mut balances = Balances::from_storage(storage);
    let mut account_balance = balances.balance(sender_canon);
    if let Some(new_balance) = account_balance.checked_add(amount) {
        account_balance = new_balance;
    } else {
        return Err(StdError::generic_err(
            "This mint attempt would increase the account's balance above the supported maximum",
        ));
    }
    balances.set_account_balance(sender_canon, account_balance);

    Ok(())
}

///
/// Updates total states to reflect balance changes
///
fn remove_balance<S: Storage>(
    storage: &mut S,
    config: &mut Config<S>,
    stake_config: &StakeConfig,
    sender: &HumanAddr,
    sender_canon: &CanonicalAddr,
    amount: u128
) -> StdResult<()> {
    // Return insufficient funds
    let mut user_shares = UserShares::may_load(
        &deps.storage,
        sender.as_str().as_bytes()
    )?.expect("No funds");

    // Get total supplied tokens
    let mut total_shares = TotalShares::load(&deps.storage)?;
    let mut total_tokens = TotalTokens::load(&deps.storage)?;

    // Calculate shares per token supplied
    let shares = shares_per_token(
        &stake_config,
        &amount.u128(),
        &total_tokens.0,
        &total_shares.0,
    );

    // Update user's shares
    if let Some(user_shares) = user_shares.0.checked_sub(shares) {
        UserShares(user_shares).save(&mut deps.storage, sender.as_str().as_bytes())?;
    }
    else {
        return Err(StdError::generic_err("Insufficient shares"))
    }

    // Update total shares
    if let Some(total) = total_shares.0.checked_sub(shares) {
        TotalShares(total).save(&mut deps.storage)?;
    }
    else {
        return Err(StdError::generic_err("Insufficient shares"))
    }

    // Update total staked
    if let Some(total) = total_tokens.0.checked_sub(amount.u128()) {
        TotalTokens(total).save(&mut deps.storage)?;
    }
    else {
        return Err(StdError::generic_err("Insufficient shares"))
    }
    if let Some(total) = config.total_supply().checked_sub(amount) {
        config.set_total_supply(total);
    }
    else {
        return Err(StdError::generic_err("Insufficient shares"))
    }

    let mut balances = Balances::from_storage(storage);
    let mut account_balance = balances.balance(sender_canon);
    if let Some(new_balance) = account_balance.checked_sub(amount) {
        account_balance = new_balance;
    } else {
        return Err(StdError::generic_err(
            "This burn attempt would decrease the account's balance to a negative",
        ));
    }
    balances.set_account_balance(sender_canon, account_balance);
    Ok(())
}

pub fn shares_per_token<S: Storage>(
    config: &StakeConfig,
    tokens: &u128,
    total_tokens: &u128,
    total_shares: &u128
) -> u128 {
    // Used to normalize the staked token to the stake token
    let token_multiplier = 10u128.pow(config.decimal_difference.into());

    if total_tokens == 0 && total_shares == 0 {
        return tokens * token_multiplier
    }

    (tokens * token_multiplier) * total_shares / (total_tokens * token_multiplier)
}

pub fn tokens_per_share<S: Storage>(
    config: &StakeConfig,
    shares: &u128,
    total_tokens: &u128,
    total_shares: &u128
) -> u128 {
    // Used to normalize the staked token to the stake token
    let token_multiplier = 10u128.pow(config.decimal_difference.into());

    if total_tokens == 0 && total_shares == 0 {
        return shares / token_multiplier
    }

    shares / (total_tokens * token_multiplier) * total_shares
}

///
/// Returns rewards in tokens, and shares
///
pub fn get_rewards(
    config: &StakeConfig,
    tokens: u128,
    shares: u128,
    total_tokens: u128,
    total_shares: u128
) -> (u128, u128) {
    let token_reward = tokens_per_share(config, &shares, &total_tokens, &total_shares) - tokens;
    (token_reward, shares_per_token(config, &token_reward, &total_tokens, &total_shares))
}

pub fn try_receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
    memo: Option<String>
) -> StdResult<HandleResponse> {

    // TODO: add a way to limit bonding if in maintenance mode

    let sender_canon = deps.api.canonical_address(&sender)?;

    let stake_config = StakeConfig::load(&deps.storage)?;

    if env.message.sender != stake_config.staked_token.address {
        return Err(StdError::generic_err("Not the stake token"))
    }

    let receive_type: ReceiveType;
    if let Some(msg) = msg {
        receive_type = from_binary(&msg)?;
    }
    else {
        return Err(StdError::generic_err("No receive type supplied in message"))
    }

    let mut config = Config::from_storage(&mut deps.storage);
    let mut messages = vec![];
    match receive_type {
        ReceiveType::Bond => {

            // Update user stake
            add_balance(
                &mut deps.storage,
                &mut config,
                &stake_config,
                &sender,
                &sender_canon,
                amount.u128()
            )?;

            // Store data
            store_stake(
                &mut deps.storage,
                &sender_canon,
                amount,
                config.constants()?.symbol,
                memo,
                &env.block
            )?;

            // Send tokens
            if let Some(treasury) = stake_config.treasury {
                messages.push(send_msg(
                    treasury,
                    amount,
                    None,
                    None,
                    None,
                    256,
                    stake_config.staked_token.code_hash,
                    stake_config.staked_token.address
                )?);
            }
            else {
                let mut stored_tokens = UnsentStakedTokens::load(&deps.storage)?;
                stored_tokens.0 += amount.u128();
                stored_tokens.save(&mut deps.storage)?;
            }
        }

        ReceiveType::Reward => {
            let mut total_tokens = TotalTokens::load(&deps.storage)?;
            total_tokens.0 += amount.u128();
            total_tokens.save(&mut deps.storage)?;

            // Store data
            store_add_reward(
                &mut deps.storage,
                &sender_canon,
                amount,
                config.constants()?.symbol,
                memo,
                &env.block
            )?;
        }

        // TODO: add Unbonding, to fund unbond amounts
        // looks for current unbond pools, if last pool has been funded
        // if fully unbonded then return remainding amount
        // also fund with unsent tokens
    };

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive { status: Success })?),
    })
}

pub fn try_unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    let sender = env.message.sender;
    let sender_canon = deps.api.canonical_address(&sender)?;

    let stake_config = StakeConfig::load(&deps.storage)?;
    let mut config = Config::from_storage(&mut deps.storage);

    // Round to that day's public unbonding queue, initialize one if empty
    let mut daily_unbond_queue = DailyUnbondingQueue::may_load(
        &deps.storage)?.unwrap_or(DailyUnbondingQueue(BinaryHeap::new()));

    let mut item_found = false;
    let day = round_date(&env.block.time);
    for &mut mut item in daily_unbond_queue.0.iter() {
        if item.release == day {
            item.unbonding += amount.u128();
            item_found = true;
            break
        }
    }
    if !item_found {
        daily_unbond_queue.0.push(DailyUnbonding::new(amount.u128(), day));
    }

    daily_unbond_queue.save(&mut deps.storage)?;

    // Check if user has an existing queue, if not, init one
    let mut unbond_queue = UnbondingQueue::may_load(
        &deps.storage, sender.as_str().as_bytes())?
        .unwrap_or(UnbondingQueue(BinaryHeap::new()));

    // Add unbonding to user queue
    unbond_queue.0.push(Unbonding { amount, release: &env.block.time + stake_config.unbond_time });

    unbond_queue.save(&mut deps.storage)?;

    // Subtract tokens from user balance
    remove_balance(&mut deps.storage, &mut config, &stake_config, &sender, &sender_canon, amount.u128())?;

    // Store the tx
    store_unbond(
        &mut deps.storage,
        &deps.api.canonical_address(&env.message.sender)?,
        amount,
        config.constants()?.symbol,
        None,
        &env.block
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Unbond { status: Success })?),
    })
}

pub fn try_claim_unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let sender = &env.message.sender;
    let stake_config = StakeConfig::load(&deps.storage)?;

    let daily_unbond_queue = DailyUnbondingQueue::load(
        &deps.storage)?.0;

    // Check if user has an existing queue, if not, init one
    let mut unbond_queue = UnbondingQueue::may_load(
        &deps.storage, sender.as_str().as_bytes())?
        .expect("No unbonding queue found");

    let mut total = Uint128::zero();
    while unbond_queue.0.peek().is_some() && &unbond_queue.0.peek().unwrap().release < &env.block.time {
        let unbond = unbond_queue.0.peek().unwrap();
        if daily_unbond_queue.iter().any(|e| e == unbond) {
            total += unbond.amount;
            unbond_queue.0.pop();
        }
    }

    unbond_queue.save(&mut deps.storage, sender.as_str().as_bytes())?;

    let messages= vec![send_msg(
        sender.clone(),
        total,
        None,
        None,
        None,
        256,
        stake_config.staked_token.code_hash,
        stake_config.staked_token.address
    )?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ClaimUnbond { status: Success })?),
    })
}

pub fn try_claim_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ClaimRewards { status: Success })?),
    })
}

pub fn try_stake_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::StakeRewards { status: Success })?),
    })
}

#[cfg(test)]
mod tests {
    use shade_protocol::shd_staking::stake::StakeConfig;
    use shade_protocol::utils::asset::Contract;
    use crate::stake::shares_per_token;

    fn init_config(token_decimals: u8, shares_decimals: u8) -> StakeConfig {
        StakeConfig {
            unbond_time: 0,
            staked_token: Contract { address: Default::default(), code_hash: "".to_string() },
            decimal_difference: shares_decimals - token_decimals,
            treasury: None
        }
    }

    #[test]
    fn tokens_per_share_test() {
        let token_decimals = 8;
        let shares_decimals = 18;
        let config = init_config(token_decimals, shares_decimals);

        let token_1 = 1 * 10u128.pow(token_decimals.into());
        let share_1 = 1 * 10u128.pow(shares_decimals.into());

        // Check for proper init when no tokens are present


        // check that tokens increase when shares decrease


        // check that tokens decrease when shares increase


    }

    #[test]
    fn shares_per_token_test() {
        let token_decimals = 8;
        let shares_decimals = 18;
        let config = init_config(token_decimals, shares_decimals);

        let token_1 = 1 * 10u128.pow(token_decimals.into());
        let share_1 = 1 * 10u128.pow(shares_decimals.into());

        // Check for proper init
        assert_eq!(shares_per_token(&config, &token_1, &0, &0), share_1);

        // Check for stability
        assert_eq!(shares_per_token(&config, &token_1, &token_1, &share_1), share_1);

        // check that shares increase when tokens decrease
        assert!(shares_per_token(&config, &token_1, &(token_1*2), &share_1) < share_1);

        // check that shares decrease when tokens increase
        assert!(shares_per_token(&config, &token_1, &token_1, &(share_1*2)) > share_1);
    }
}