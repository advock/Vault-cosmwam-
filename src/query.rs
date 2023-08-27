use cosmwasm_std::{
    Addr, Deps, DepsMut, Int128, StdError, StdResult, Uint128,
};

use crate::contract::BASIS_POINTS_DIVISOR;
use crate::state::{
    Config, Position, CONFIG, ISMANAGER, POSITION, WHITELISTEDTOKEN,
};

use crate::{helpers::validate};

pub fn query_config(_deps: Deps) -> StdResult<Config> {
    let res = CONFIG.may_load(_deps.storage)?;

    match res {
        Some(val) => Ok(val),
        None => Err(StdError::NotFound {
            kind: format!("Unable to load token state"),
        }),
    }
}

pub fn query_manager(_deps: Deps, address: Addr) -> StdResult<bool> {
    let res = ISMANAGER.may_load(_deps.storage, address)?;

    match res {
        Some(val) => Ok(val),
        None => Err(StdError::NotFound {
            kind: format!("Unable to load token state"),
        }),
    }
}

pub fn check_whitelisted_token(_deps: Deps, address: Addr) -> StdResult<bool> {
    let res = WHITELISTEDTOKEN.may_load(_deps.storage, address)?;
    match res {
        Some(val) => Ok(val),
        None => Err(StdError::NotFound {
            kind: format!("Unable to load token state"),
        }),
    }
}

pub fn all_whiteListed_token(_deps: Deps) -> StdResult<Vec<Addr>> {
    let res = CONFIG.may_load(_deps.storage)?;

    match res {
        Some(val) => Ok(val.all_whitelisted_tokens),
        None => Err(StdError::NotFound {
            kind: format!("Unable to load token state"),
        }),
    }
}

pub fn get_position(_deps: Deps, key: Vec<u8>) -> StdResult<Position> {
    let res = POSITION.may_load(_deps.storage, key)?;

    match res {
        Some(_val) => Ok(res.unwrap()),
        None => Ok(Position {
            size: Uint128::zero(),                 // Set default value for Uint256
            collateral: Uint128::zero(),           // Set default value for Uint256
            averagePrice: Uint128::zero(),         // Set default value for Uint256
            entryFundingRate: Uint128::zero(),     // Set default value for Uint256
            reserveAmount: Uint128::zero(),        // Set default value for Uint256
            realisedPnL: Int128::zero(),           // Set default value for Int256
            lastIncreasedTime: Default::default(), // Set default value for Uint256
        }),
    }
}

pub fn get_position_key(
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
) -> Vec<u8> {
    // Convert addresses to bytes
    let account_bytes = account.as_str().as_bytes();
    let collateral_token_bytes = collateral_token.as_str().as_bytes();
    let index_token_bytes = index_token.as_str().as_bytes();

    // Calculate the size of the resulting key
    let key_size = account_bytes.len() + collateral_token_bytes.len() + index_token_bytes.len() + 1;

    // Initialize the key vector with the correct size
    let mut key = Vec::with_capacity(key_size);

    // Extend the key with the bytes of addresses and the boolean
    key.extend_from_slice(account_bytes);
    key.extend_from_slice(collateral_token_bytes);
    key.extend_from_slice(index_token_bytes);
    key.push(if is_long { 1 } else { 0 });

    key
}

pub fn get_position_leverage(
    _deps: DepsMut,
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
) -> StdResult<Uint128> {
    let key = get_position_key(account, collateral_token, index_token, is_long);

    let position = POSITION.load(_deps.storage, key)?;
    validate(position.collateral > Uint128::zero(), "err");

    let res: Uint128 = position.size * BASIS_POINTS_DIVISOR / position.collateral;

    Ok(res)
}
