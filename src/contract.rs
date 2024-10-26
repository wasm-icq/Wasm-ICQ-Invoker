use cosmwasm_std::{Binary, Coin, Deps, DepsMut, Env, IbcMsg, MessageInfo, Response, StdError, StdResult, to_json_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, IbcRegisterBalanceQuery, InstantiateMsg, QueryBalanceMsg, QueryMsg};
use crate::state::{CHANNEL_INFO, ICQ_RESPONSES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:wasm-icq-invoker";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.debug("WASMDEBUG: instantiate");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SendQueryBalance(msg) => send_query_balance(deps, env, msg),
    }
}

pub fn send_query_balance(
    deps: DepsMut,
    env: Env,
    msg: QueryBalanceMsg,
) -> Result<Response, ContractError> {
    let channel_id: String = get_channel_id(deps.as_ref())?;

    let packet_data: IbcRegisterBalanceQuery = IbcRegisterBalanceQuery {
        chain_id: msg.chain_id,
        addr: msg.addr,
        denom: msg.denom,
    };

    // timeout is in nanoseconds
    let timeout = env.block.time.plus_seconds(120);

    // prepare ibc message
    let ibc_msg = IbcMsg::SendPacket {
        channel_id: channel_id.clone(),
        data: to_json_binary(&packet_data)?,
        timeout: timeout.into(),
    };

    Ok(Response::new()
        .add_attribute("method", "send_query_balance")
        .add_attribute("channel", channel_id)
        .add_message(ibc_msg))
}

fn get_channel_id(deps: Deps) -> StdResult<String> {
    match CHANNEL_INFO.may_load(deps.storage)? {
        Some(channel_info) => Ok(channel_info.id), // Return the item if it's loaded
        None => Err(StdError::generic_err("Channel to ICQ module is not setup")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllBalances {} => to_json_binary(&query_all_balances(deps)?),
    }
}

fn query_all_balances(deps: Deps) -> StdResult<Vec<(u64, Coin)>> {
    let balances: StdResult<Vec<(u64, Coin)>> = ICQ_RESPONSES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect();

    // Convert the result to binary
    balances
}

#[cfg(test)]
mod tests {}
