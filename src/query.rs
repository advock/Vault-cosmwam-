use cosmwasm_std::{
    Addr, Binary, Deps, DepsMut, Env, Int128, MessageInfo, Response, StdError, StdResult, Uint128,
};

use crate::state::{Config, Position, CONFIG, ISMANAGER, POSITION, WHITELISTEDTOKEN};

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
        Some(val) => Ok(res.unwrap()),
        None => Ok((Position {
            size: Uint128::zero(),                 // Set default value for Uint256
            collateral: Uint128::zero(),           // Set default value for Uint256
            averagePrice: Uint128::zero(),         // Set default value for Uint256
            entryFundingRate: Uint128::zero(),     // Set default value for Uint256
            reserveAmount: Uint128::zero(),        // Set default value for Uint256
            realisedPnL: Int128::zero(),           // Set default value for Int256
            lastIncreasedTime: Default::default(), // Set default value for Uint256
        })),
    }
}
