use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, DepsMut, Env, Event, MessageInfo, QueryRequest, Response,
    StdError, StdResult, Uint128, Uint256, WasmMsg, WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

use crate::{
    msg::ExecuteMsg,
    state::{MAXUSDGAMOUNT, POOLAMOUNT, TOKENBALANCE, USDGAMOUNT},
    ContractError,
};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

pub fn validate(condition: bool, error_message: &str) -> StdResult<()> {
    if condition {
        Ok(())
    } else {
        Err(StdError::GenericErr {
            msg: error_message.to_string(),
        })
    }
}

pub fn _increaseUsdgAmount(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _amount: Uint256,
) -> Result<Response, ContractError> {
    let mut usdgamount = USDGAMOUNT.load(_deps.storage, _token.clone())?;

    USDGAMOUNT.save(_deps.storage, _token.clone(), &(usdgamount + _amount))?;

    let mut maxUsdgAmount = MAXUSDGAMOUNT.load(_deps.storage, _token.clone())?;

    if maxUsdgAmount != Uint256::zero() {
        validate(usdgamount <= maxUsdgAmount, "err")?;
    };

    let event = Event::new("IncreaseUsdgAmount")
        .add_attribute("token", _token.as_str())
        .add_attribute("amount", _amount.to_string());

    let res = Response::new().add_event(event);
    Ok(res)
}

pub fn _decreaseUsdgAmount(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _amount: Uint256,
) -> Result<Response, ContractError> {
    let mut usdgamount = USDGAMOUNT.load(_deps.storage, _token.clone())?;

    if usdgamount < _amount {
        USDGAMOUNT.save(_deps.storage, _token.clone(), &Uint256::zero())?;
        let event = Event::new("DecreaseUsdgAmount")
            .add_attribute("token", _token.as_str())
            .add_attribute("amount", _amount.to_string());
        return Ok(Response::new().add_event(event));
    } else {
    }

    USDGAMOUNT.save(_deps.storage, _token.clone(), &(usdgamount - _amount))?;

    let event = Event::new("DecreaseUsdgAmount")
        .add_attribute("token", _token.as_str())
        .add_attribute("amount", _amount.to_string());

    let res = Response::new().add_event(event);
    Ok(res)
}

pub fn transfer_cw20_tokens(
    contract_address: Addr,
    sender_address: Addr,
    recipient_address: Addr,
    amount: Uint128,
) -> Result<CosmosMsg, ContractError> {
    let transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient: recipient_address.into_string(),
        amount,
    };

    let exec_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_address.to_string(),
        msg: to_binary(&transfer_msg).map_err(|e| e)?,
        funds: vec![],
    });

    Ok(exec_msg)
}

pub fn balance_cw20_tokens(
    _deps: &DepsMut,
    _env: Env,
    contract_address: Addr,
) -> Result<Uint128, ContractError> {
    let prev_balance = TOKENBALANCE.load(_deps.storage, contract_address.clone())?;

    let query_msg = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_address.clone().into_string(),
        msg: to_binary(&cw20::Cw20QueryMsg::Balance {
            address: _env.contract.address.clone().into_string(),
        })?,
    });

    let query_result: BalanceResponse = _deps.querier.query(&query_msg)?;

    let balance = query_result.balance;
    Ok(balance)
}

pub fn _increasePoolAmount(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    let poolAmount = POOLAMOUNT.load(_deps.storage, _token.clone())?;
    POOLAMOUNT.save(_deps.storage, _token.clone(), &(poolAmount + _amount))?;

    let balance = balance_cw20_tokens(&_deps, _env, _token.clone())?;
    let poolAmount_next = POOLAMOUNT.load(_deps.storage, _token.clone())?;
    validate(poolAmount <= balance, "error_message")?;

    let event = Event::new("IncreasePoolAmount")
        .add_attribute("token", _token.as_str())
        .add_attribute("amount", _amount.to_string());

    Ok(Response::new().add_event(event))
}

pub fn updateCumulativeFundingRate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<bool, ContractError> {
    Ok(true)
}
