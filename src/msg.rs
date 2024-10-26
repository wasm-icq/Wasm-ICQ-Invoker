use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SendQueryBalance(QueryBalanceMsg),
}

#[cw_serde]
pub struct QueryBalanceMsg {
    pub chain_id: String,
    pub addr: String,
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    AllBalances {},
}

#[cw_serde]
pub struct IbcRegisterBalanceQuery {
    pub chain_id: String,
    pub addr: String,
    pub denom: String,
}

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct BalanceResponse {
    pub balances: Balances,
    pub last_submitted_local_height: u64,
}

#[cw_serde]
pub struct Balances {
    pub coins: Vec<Coin>,
}