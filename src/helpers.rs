use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Binary, ContractResult, CosmosMsg, DepsMut, Env, Event, MessageInfo,
    QueryRequest, Response, StdError, StdResult, Uint128, Uint256, WasmMsg, WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

use crate::{
    msg::ExecuteMsg,
    state::{
        CONFIG, FEERESERVED, GLOBALSHORTAVERAGEPRICE, GLOBALSHORTSIZE, GUARANTEEUSD, MAXUSDGAMOUNT,
        POOLAMOUNT, RESERVEDAMOUNTS, SHORTABLETOKEN, STABLETOKEN, TOKENBALANCE, TOKENDECIMAL,
        USDGAMOUNT, WHITELISTEDTOKEN,
    },
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
    _amount: Uint128,
) -> Result<Response, ContractError> {
    let mut usdgamount = USDGAMOUNT.load(_deps.storage, _token.clone())?;

    USDGAMOUNT.save(_deps.storage, _token.clone(), &(usdgamount + _amount))?;

    let mut maxUsdgAmount = MAXUSDGAMOUNT.load(_deps.storage, _token.clone())?;

    if maxUsdgAmount != Uint128::zero() {
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
    _amount: Uint128,
) -> Result<Response, ContractError> {
    let mut usdgamount = USDGAMOUNT.load(_deps.storage, _token.clone())?;

    if usdgamount < _amount {
        USDGAMOUNT.save(_deps.storage, _token.clone(), &Uint128::zero())?;
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

pub fn get_min_price(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
) -> Result<Uint128, ContractError> {
    unimplemented!()
}
pub fn get_max_price(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
) -> Result<Uint128, ContractError> {
    unimplemented!()
}

pub fn getBuyUsdgFeeBasisPoints(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    usdgAmount: Uint128,
) -> Result<Uint128, ContractError> {
    unimplemented!()
}
pub fn getSellUsdgFeeBasisPoints(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    usdgAmount: Uint128,
) -> Result<Uint128, ContractError> {
    unimplemented!()
}

pub fn getSwapFeeBasisPoints(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _tokenin: Addr,
    _tokenout: Addr,
    usdgAmount: Uint128,
) -> Result<Uint128, ContractError> {
    unimplemented!()
}

pub fn _decreasePoolAmount(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    let poolAmount = POOLAMOUNT.load(_deps.storage, _token.clone())?;
    POOLAMOUNT.save(_deps.storage, _token.clone(), &(poolAmount - _amount))?;

    let balance = RESERVEDAMOUNTS.load(_deps.storage, _token.clone())?;
    let poolAmount_next = POOLAMOUNT.load(_deps.storage, _token.clone())?;
    validate(balance <= poolAmount_next, "error_message")?;

    let event = Event::new("DecreasePoolAmount")
        .add_attribute("token", _token.as_str())
        .add_attribute("amount", _amount.to_string());

    Ok(Response::new().add_event(event))
}

pub fn update_token_bal(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _address: Addr,
) -> Result<Response, ContractError> {
    let bal: Uint128 = balance_cw20_tokens(&_deps, _env, _address.clone())?;
    TOKENBALANCE.save(_deps.storage, _address, &bal);
    Ok(Response::new())
}

pub fn _validateTokens(
    _deps: &DepsMut,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
) -> StdResult<()> {
    if is_long {
        validate(
            collateral_token == index_token,
            "ERR_COLLATERAL_INDEX_MISMATCH",
        )?;
        let whitelisted_tokens = WHITELISTEDTOKEN.load(_deps.storage, collateral_token)?;
        validate(whitelisted_tokens, "ERR_COLLATERAL_NOT_WHITELISTED")?;
        let stable_tokens = STABLETOKEN.load(_deps.storage, collateral_token)?;

        validate(!stable_tokens, "ERR_COLLATERAL_STABLE")?;
        return Ok(());
    } else {
        let whitelisted_tokens = WHITELISTEDTOKEN.load(_deps.storage, collateral_token)?;
        validate(whitelisted_tokens, "ERR_COLLATERAL_NOT_WHITELISTED")?;
        let stable_tokens = STABLETOKEN.load(_deps.storage, collateral_token)?;

        validate(!stable_tokens, "ERR_COLLATERAL_STABLE")?;

        let shortable_token = SHORTABLETOKEN.load(_deps.storage, collateral_token)?;
        validate(shortable_token, "ERR_COLLATERAL_STABLE")?;

        let stable_tokens = STABLETOKEN.load(_deps.storage, index_token)?;

        validate(!stable_tokens, "ERR_COLLATERAL_STABLE")?;

        Ok(())
    }
}

pub fn get_next_average_price(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _index_token: &Addr,
    _size: Uint128,
    _average_price: Uint128,
    _is_long: bool,
    _next_price: Uint128,
    _size_delta: Uint128,
    _last_increased_time: u64,
) -> Result<Uint128, ContractError> {
    let (has_profit, delta) = get_delta(
        _deps,
        _env,
        _info,
        *_index_token,
        _size,
        _average_price,
        _is_long,
        _last_increased_time,
    )?;

    let next_size = _size + _size_delta;
    let divisor = if _is_long {
        if has_profit {
            next_size + delta
        } else {
            next_size - delta
        }
    } else {
        if has_profit {
            next_size - delta
        } else {
            next_size + delta
        }
    };

    let next_average_price: Uint128 = _next_price * next_size / divisor;

    Ok(next_average_price)
}

pub fn get_delta(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _index_token: Addr,
    _size: Uint128,
    _average_price: Uint128,
    _is_long: bool,
    _last_increased_time: u64,
) -> Result<(bool, Uint128), ContractError> {
    validate(_average_price > Uint128::zero(), "ERR_AVERAGE_PRICE_ZERO")?;

    let price = if _is_long {
        get_min_price(_deps, _env, _info, _index_token)?
    } else {
        get_max_price(_deps, _env, _info, _index_token)?
    };

    let price_delta = if _average_price > price {
        _average_price - price
    } else {
        price - _average_price
    };

    let delta: Uint128 = _size * price_delta / _average_price;

    let has_profit = if _is_long {
        price > _average_price
    } else {
        _average_price > price
    };

    // Define constants for BASIS_POINTS_DIVISOR and minProfitTime
    const BASIS_POINTS_DIVISOR: Uint128 = Uint128::new(10000);
    const MIN_PROFIT_TIME: u64 = 3600; // Placeholder value in seconds

    // Placeholder for minProfitBasisPoints
    let min_profit_basis_points: Uint128 = Uint128::new(500); // Placeholder value

    let min_bps: Uint128 =
        if (_env.block.time.seconds() as u64 > _last_increased_time + MIN_PROFIT_TIME) {
            Uint128::zero()
        } else {
            min_profit_basis_points
        };

    if has_profit && delta * BASIS_POINTS_DIVISOR <= min_bps {
        return Ok((has_profit, Uint128::zero()));
    }

    Ok((has_profit, delta))
}

pub fn get_position_fee(
    _account: Addr,
    _collateral_token: Addr,
    _index_token: Addr,
    _is_long: bool,
    _size_delta: u128,
) -> Uint128 {
    // Your implementation here
    unimplemented!()
}

// Placeholder for getFundingFee implementation
pub fn get_funding_fee(
    _account: Addr,
    _collateral_token: Addr,
    _index_token: Addr,
    _is_long: bool,
    _size: u128,
    _entry_funding_rate: u128,
) -> Uint128 {
    // Your implementation here
    unimplemented!()
}

pub fn usd_to_token_min(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _usd_amount: u128,
) -> StdResult<u128> {
    if _usd_amount == 0 {
        return Ok(0);
    }
    let max_price = get_max_price(deps, _env, _info, _token).unwrap();
    let result = usd_to_token(deps, &_token, _usd_amount, max_price.u128())?;
    Ok(result)
}

pub fn usd_to_token(
    deps: DepsMut,
    _token: &Addr,
    _usd_amount: u128,
    _price: u128,
) -> StdResult<u128> {
    if _usd_amount == 0 {
        return Ok(0);
    }
    let decimals: u32 = 6;
    let result: u128 = _usd_amount * 10u128.pow(decimals) / _price;
    Ok(result)
}

pub fn _collect_margin_fees(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _account: Addr,
    _collateral_token: Addr,
    _index_token: Addr,
    _is_long: bool,
    _size_delta: u128,
    _size: u128,
    _entry_funding_rate: u128,
) -> Result<Response, ContractError> {
    let mut feeUsd = get_position_fee(
        _account,
        _collateral_token,
        _index_token,
        _is_long,
        _size_delta,
    );
    let fundingFee = get_funding_fee(
        _account,
        _collateral_token,
        _index_token,
        _is_long,
        _size,
        _entry_funding_rate,
    );

    feeUsd = feeUsd + fundingFee;

    let feeTokens: Uint128 = Uint128::new(usd_to_token_min(
        deps,
        _env,
        _info,
        _collateral_token,
        feeUsd.u128(),
    )?);

    let feeReserves = FEERESERVED.load(deps.storage, _collateral_token)?;

    FEERESERVED.save(deps.storage, _collateral_token, &(feeReserves + feeTokens));

    let event = Event::new("collect_margin_fees")
        .add_attribute("collateral_token", _collateral_token.as_str())
        .add_attribute("fee_usd", feeUsd.to_string())
        .add_attribute("fee_tokens", feeTokens.to_string());

    Ok(Response::new()
        .add_attribute("feeReserves", feeReserves.to_string())
        .add_event(event))
}

pub fn token_to_usd_min(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _token_amount: u128,
) -> Result<Uint128, ContractError> {
    if _token_amount == 0 {
        return Ok(Uint128::zero());
    }
    let price: Uint128 = get_min_price(deps, _env, _info, _token)?;
    let decimals = get_max_price(deps, _env, _info, _token)?;
    let result: u128 = _token_amount * price.u128() / 10u128.pow(decimals.u128() as u32);
    Ok(Uint128::new(result))
}

pub fn get_entry_funding_rate(
    deps: DepsMut,
    _collateral_token: Addr,
    _index_token: Addr,
    _is_long: bool,
) -> StdResult<Uint128> {
    unimplemented!()
}

pub fn usdToTokenMax(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _usdAmount: Uint128,
) -> Result<Uint128, ContractError> {
    if (_usdAmount == Uint128::zero()) {
        return Ok(Uint128::zero());
    } else {
        let price = get_min_price(_deps, _env, _info, _token)?;
        let res = usd_to_token(_deps, &_token, _usdAmount.u128(), price.u128())?;
        Ok(Uint128::new(res))
    }
}

pub fn _increaseReservedAmount(
    deps: DepsMut,
    _collateral_token: Addr,
    reserveDelta: Uint128,
) -> Result<Response, ContractError> {
    unimplemented!()
}

pub fn _increaseGuaranteedUsd(
    deps: DepsMut,
    _collateral_token: Addr,
    _usdamount: Uint128,
) -> Result<Response, ContractError> {
    let mut guaranteedUsd = GUARANTEEUSD.load(deps.storage, _collateral_token)?;
    guaranteedUsd = guaranteedUsd + _usdamount;

    GUARANTEEUSD.save(deps.storage, _collateral_token, &guaranteedUsd);
    let mut response = Response::new();
    let event =
        Event::new("_increaseGuaranteedUsd").add_attribute("token", _collateral_token.to_string());

    Ok(response.add_event(event))
}

pub fn _decreaseGuaranteedUsd(
    deps: DepsMut,
    _collateral_token: Addr,
    _usdamount: Uint128,
) -> Result<Response, ContractError> {
    let mut guaranteedUsd = GUARANTEEUSD.load(deps.storage, _collateral_token)?;
    guaranteedUsd = guaranteedUsd - _usdamount;

    GUARANTEEUSD.save(deps.storage, _collateral_token, &guaranteedUsd);
    let mut response = Response::new();
    let event =
        Event::new("_decreaseGuaranteedUsd").add_attribute("token", _collateral_token.to_string());

    Ok(response.add_event(event))
}

pub fn get_next_global_short_average_price(
    deps: DepsMut,
    _index_token: Addr,
    _next_price: Uint128,
    _size_delta: Uint128,
) -> Result<Uint128, ContractError> {
    let size = GLOBALSHORTSIZE.load(deps.storage, _index_token)?;

    let average_price = GLOBALSHORTAVERAGEPRICE.load(deps.storage, _index_token)?;

    let price_delta = if average_price > _next_price {
        average_price - _next_price
    } else {
        _next_price - average_price
    };
    let delta = size * price_delta / average_price;
    let has_profit = average_price > _next_price;

    let next_size = size + _size_delta;
    let divisor = if has_profit {
        next_size - delta
    } else {
        next_size + delta
    };

    let result = _next_price * next_size / divisor;
    Ok(result)
}

pub fn _decreaseReservedAmount(
    deps: DepsMut,
    _token: Addr,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    let mut guaranteedUsd = RESERVEDAMOUNTS.load(deps.storage, _token)?;
    guaranteedUsd = guaranteedUsd - _amount;

    RESERVEDAMOUNTS.save(deps.storage, _token, &guaranteedUsd);
    let mut response = Response::new();
    let event = Event::new("_decreaseReservedAmount").add_attribute("token", _token.to_string());

    Ok(response.add_event(event))
}

pub fn _decreaseGlobalShortSize(
    _deps: DepsMut,
    _token: Addr,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    let size = GLOBALSHORTSIZE.load(_deps.storage, _token)?;
    if _amount > size {
        GLOBALSHORTSIZE.save(_deps.storage, _token, &Uint128::zero());
        return Ok(Response::new());
    } else {
        GLOBALSHORTSIZE.save(_deps.storage, _token, &(size - _amount))?;
        Ok(Response::new())
    }
}

pub fn validLiquidation(
    mut _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _account: Addr,
    _collateralToken: Addr,
    _indexToken: Addr,
    _isLong: bool,
    _receiver: Addr,
) -> Result<(Uint128, Uint128), ContractError> {
    unimplemented!()
}

pub fn getRedemptionCollateral(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
) -> Result<Uint128, ContractError> {
    let stableTokens = STABLETOKEN.load(_deps.storage, _token)?;
    let amount = POOLAMOUNT.load(_deps.storage, _token)?;
    if stableTokens {
        return Ok(amount);
    }
    let grantedusd = GUARANTEEUSD.load(_deps.storage, _token)?;

    let collateral = usd_to_token_min(_deps, _env, _info, _token, grantedusd.u128())?;

    let _collateral = Uint128::new(collateral);

    let reservedAmounts = RESERVEDAMOUNTS.load(_deps.storage, _token)?;
    let res: Uint128 = _collateral + amount - reservedAmounts;

    Ok(res)
}

pub fn getRedemptionCollateralUsd(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
) -> Result<Uint128, ContractError> {
    let RedemptionCollateral = getRedemptionCollateral(_deps, _env, _info, _token)?;

    let res = token_to_usd_min(_deps, _env, _info, _token, RedemptionCollateral.u128())?;
    Ok(res)
}

pub fn getPositionFee(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _account: Addr,
    _collateralToken: Addr,
    _indexToken: Addr,
    _isLong: bool,
    _sizeDelta: Uint128,
) -> Result<Uint128, ContractError> {
    unimplemented!()
}

pub fn getFeeBasisPoints(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _usdgDelta: Uint128,
    _feeBasisPoints: Uint128,
    _taxBasisPoints: Uint128,
    _increment: bool,
) -> Result<Uint128, ContractError> {
    unimplemented!()
}
