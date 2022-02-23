use std::collections::BinaryHeap;
use cosmwasm_std::{HumanAddr, Uint128};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use shade_protocol::shd_staking::stake::{DailyUnbonding, Unbonding};
use shade_protocol::storage::{BucketStorage, SingletonStorage};

// used to determine what each token is worth to calculate rewards
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TotalShares(pub Uint128);

impl SingletonStorage for TotalShares {
    const NAMESPACE: &'static [u8] = b"total_shares";
}

// used to separate tokens minted from total tokens (includes rewards)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TotalTokens(pub Uint128);

impl SingletonStorage for TotalTokens {
    const NAMESPACE: &'static [u8] = b"total_tokens";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UserShares(pub Uint128);

impl BucketStorage for UserShares {
    const NAMESPACE: &'static [u8] = b"user_shares";
}

// stores received token info if no treasury is set
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnsentStakedTokens(pub Uint128);

impl SingletonStorage for UnsentStakedTokens {
    const NAMESPACE: &'static [u8] = b"unsent_staked_tokens";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TotalUnbonding(pub Uint128);

impl SingletonStorage for TotalUnbonding {
    const NAMESPACE: &'static [u8] = b"total_unbonding";
}

// used to check if unbonding id funded
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnbondingIsFunded(pub Uint128);

impl SingletonStorage for UnbondingIsFunded {
    const NAMESPACE: &'static [u8] = b"unbonding_is_funded";
}

// Distributors wrappers

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Distributors(pub Vec<HumanAddr>);

impl SingletonStorage for Distributors {
    const NAMESPACE: &'static [u8] = b"distributors";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DistributorsEnabled(pub bool);

impl SingletonStorage for DistributorsEnabled {
    const NAMESPACE: &'static [u8] = b"distributors_transfer";
}

// Unbonding Queues

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnbondingQueue(pub BinaryHeap<Unbonding>);

impl BucketStorage for UnbondingQueue {
    const NAMESPACE: &'static [u8] = b"unbonding_queue";
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DailyUnbondingQueue(pub BinaryHeap<DailyUnbonding>);

impl SingletonStorage for DailyUnbondingQueue {
    const NAMESPACE: &'static [u8] = b"daily_unbonding_queue";
}