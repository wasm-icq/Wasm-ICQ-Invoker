use cosmwasm_std::{DepsMut, Env, from_json, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{ContractError, error::Never};
use crate::ack::{make_ack_fail, make_ack_success};
use crate::msg::{BalanceResponse, Balances};
use crate::state::{CHANNEL_INFO, ChannelInfo, ICQ_RESPONSES};

pub const IBC_VERSION: &str = "icq-1";

/// Handles the `OpenInit` and `OpenTry` parts of the IBC handshake.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    deps.api.debug("WASMDEBUG: ibc_channel_connect");
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;

    let channel: IbcChannel = msg.into();
    let info = ChannelInfo {
        id: channel.endpoint.channel_id,
        counterparty_endpoint: channel.counterparty_endpoint,
        connection_id: channel.connection_id,
    };
    CHANNEL_INFO.save(deps.storage, &info)?;

    Ok(IbcBasicResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    deps.api.debug("WASMDEBUG: ibc_channel_close");
    let channel = msg.channel().endpoint.channel_id.clone();
    // Reset the state for the channel.
    CHANNEL_INFO.remove(deps.storage);
    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_close")
        .add_attribute("channel", channel))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    deps.api.debug("WASMDEBUG: ibc_packet_receive");
    let packet = msg.packet;

    do_ibc_packet_receive(deps, &packet).or_else(|err| {
        Ok(IbcReceiveResponse::new(make_ack_fail(err.to_string()))
            .add_attribute("action", "ibc_packet_receive")
            .add_attribute("success", "false")
            .add_attribute("error", err.to_string())
        )
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    deps.api.debug("WASMDEBUG: ibc_packet_ack");
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_ack"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_timeout"))
}

pub fn validate_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    // We expect an unordered channel here. Ordered channels have the
    // property that if a message is lost the entire channel will stop
    // working until you start it again.
    if channel.order != IbcOrder::Unordered {
        return Err(ContractError::OnlyOrderedChannel {});
    }

    if channel.version != IBC_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            actual: channel.version.to_string(),
            expected: IBC_VERSION.to_string(),
        });
    }

    // Make sure that we're talking with a counterparty who speaks the
    // same "protocol" as us.
    //
    // For a connection between chain A and chain B being established
    // by chain A, chain B knows counterparty information during
    // `OpenTry` and chain A knows counterparty information during
    // `OpenAck`. We verify it when we have it but when we don't it's
    // alright.
    if let Some(counterparty_version) = counterparty_version {
        if counterparty_version != IBC_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                actual: counterparty_version.to_string(),
                expected: IBC_VERSION.to_string(),
            });
        }
    }

    Ok(())
}

fn do_ibc_packet_receive(
    deps: DepsMut,
    packet: &IbcPacket,
) -> Result<IbcReceiveResponse, ContractError> {
    let balance_response: BalanceResponse = from_json(&packet.data)?;

    let balances: Balances = balance_response.balances;
    let balance = balances.coins.first().unwrap();

    ICQ_RESPONSES.save(deps.storage, packet.sequence, &balance)?;

    Ok(IbcReceiveResponse::new(make_ack_success())
        .add_attribute("method", "ibc_packet_receive")
        .add_attribute("sequence", packet.sequence.to_string())
    )
}