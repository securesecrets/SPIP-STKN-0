use cosmwasm_std::{Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage, to_binary, Uint128};
use shade_protocol::shd_staking::stake::StakeConfig;
use shade_protocol::storage::{BucketStorage, SingletonStorage};
use crate::msg::QueryAnswer;
use crate::stake::{calculate_rewards, shares_per_token};
use crate::state::{ReadonlyBalances};
use crate::state_staking::{DailyUnbondingQueue, TotalShares, TotalTokens, UnbondingQueue, UserShares};

pub fn stake_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {

    to_binary(&QueryAnswer::StakedConfig { config: StakeConfig::load(&deps.storage)? })
}

pub fn total_staked<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {

    to_binary(&QueryAnswer::TotalStaked {
        tokens: Uint128(TotalTokens::load(&deps.storage)?.0),
        shares: Uint128(TotalShares::load(&deps.storage)?.0)
    })
}

pub fn stake_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {

    to_binary(&QueryAnswer::StakeRate {
        shares: Uint128(shares_per_token(
            &StakeConfig::load(&deps.storage)?,
            &1,
            &TotalTokens::load(&deps.storage)?.0,
            &TotalShares::load(&deps.storage)?.0
        ))
    })
}

pub fn unbonding<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start: u64,
    end: u64
) -> StdResult<Binary> {

    let mut total = Uint128::zero();

    let mut queue = DailyUnbondingQueue::load(&deps.storage)?.0;

    while !queue.is_empty() {
        let item = queue.pop().unwrap();
        if item.release > start {
            if item.release > end {
                break
            }
            total += Uint128(item.unbonding);
        }
    }

    to_binary(&QueryAnswer::Unbonding {
        total
    })
}
pub fn staked<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: HumanAddr,
    time: Option<u64>
) -> StdResult<Binary> {

    let tokens = ReadonlyBalances::from_storage(&deps.storage).account_amount(
        &deps.api.canonical_address(&account)?
    );

    let shares = UserShares::load(
        &deps.storage,
        account.as_str().as_bytes()
    )?.0;

    let (rewards, _) = calculate_rewards(
        &StakeConfig::load(&deps.storage)?,
        tokens,
        shares,
        TotalTokens::load(&deps.storage)?.0,
        TotalShares::load(&deps.storage)?.0
    );

    let mut queue = UnbondingQueue::load(
        &deps.storage,
        account.as_str().as_bytes()
    )?.0;

    let mut unbonding = Uint128::zero();
    let mut unbonded = Uint128::zero();

    while !queue.is_empty() {
        let item = queue.pop().unwrap();
        if let Some(time) = time {
            if item.release <= time {
                unbonded += item.amount;
            }
            else {
                unbonding += item.amount;
            }
        }
        else {
            unbonding += item.amount;
        }
    }

    to_binary(&QueryAnswer::Staked {
        tokens: Uint128(tokens),
        shares: Uint128(shares),
        pending_rewards: Uint128(rewards),
        unbonding,
        unbonded: match time {
            None => None,
            Some(_) => Some(unbonded)
        }
    })
}