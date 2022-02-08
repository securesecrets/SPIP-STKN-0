use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdResult, Storage, to_binary};
use shade_protocol::shd_staking::stake::{Distributors, DistributorsEnabled};
use shade_protocol::storage::SingletonStorage;
use crate::contract::check_if_admin;
use crate::msg::HandleAnswer;
use crate::msg::ResponseStatus::Success;
use crate::state::Config;

pub fn get_distributor<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Option<Vec<HumanAddr>>>{
    Ok(match DistributorsEnabled::load(&deps.storage)?.0 {
        true => Some(Distributors::load(&deps.storage)?.0),
        false => None
    })
}


pub fn try_add_distributors<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_distributors: Vec<HumanAddr>
) -> StdResult<HandleResponse> {
    let mut config = Config::from_storage(&mut deps.storage);

    check_if_admin(&config, &env.message.sender)?;

    let mut distributors = Distributors::load(&mut deps.storage)?;
    distributors.0.extend(new_distributors);
    distributors.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddDistributors { status: Success })?),
    })
}

pub fn try_set_distributors<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    distributors: Vec<HumanAddr>
) -> StdResult<HandleResponse> {
    let mut config = Config::from_storage(&mut deps.storage);

    check_if_admin(&config, &env.message.sender)?;

    Distributors(distributors).save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetDistributors { status: Success })?),
    })
}