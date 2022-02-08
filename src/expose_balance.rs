use cosmwasm_std::{Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier, StdResult, Storage, to_binary, Uint128};
use secret_toolkit::utils::HandleCallback;
use crate::contract::check_if_admin;
use crate::msg::HandleAnswer;
use crate::msg::ResponseStatus::Success;
use crate::state::Config;

pub fn try_expose_balance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,

) -> StdResult<HandleResponse> {



    let message = vec![];

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