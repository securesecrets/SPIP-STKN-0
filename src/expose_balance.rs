use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary, Uint128};
use secret_toolkit::utils::HandleCallback;
use crate::contract::check_if_admin;
use crate::msg::HandleAnswer;
use crate::msg::ResponseStatus::Success;
use crate::state::{Balances, Config, get_receiver_hash};

pub fn try_expose_balance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
    code_hash: String,
    msg: Option<Binary>,
    memo: Option<String>
) -> StdResult<HandleResponse> {

    // Get balance to expose
    let balance = Balances::from_storage(&mut deps.storage)
        .balance(&deps.api.canonical_address(&env.message.sender)?);

    let receiver_hash: String;
    if let Some(code_hash) = code_hash {
        receiver_hash = code_hash;
    }
    else if let Some(code_hash) = get_receiver_hash(&deps.storage, &recipient) {
        receiver_hash = code_hash?;
    }
    else {
        return Err(StdError::generic_err("No code hash received"))
    }

    let messages = vec![Snip20BalanceReceiverMsg::new(
        env.message.sender,
        balance,
        memo,
        msg
    ).to_cosmos_msg(receiver_hash, recipient)?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExposeBalance { status: Success })?),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Snip20BalanceReceiverMsg {
    pub sender: HumanAddr,
    pub balance: Uint128,
    pub memo: Option<String>,
    pub msg: Option<Binary>,
}

impl Snip20BalanceReceiverMsg {
    pub fn new(
        sender: HumanAddr,
        balance: u128,
        memo: Option<String>,
        msg: Option<Binary>,
    ) -> Self {
        Self {
            sender,
            balance: Uint128(balance),
            memo,
            msg
        }
    }

    pub fn to_cosmos_msg(
        &self,
        code_hash: String,
        address: HumanAddr
    ) -> StdResult<CosmosMsg> {
        self.to_cosmos_msg(code_hash, address)
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BalanceReceiverHandleMsg {
    ReceiveBalance(Snip20BalanceReceiverMsg)
}

impl HandleCallback for BalanceReceiverHandleMsg {
    const BLOCK_SIZE: usize = 256;
}