use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

use crate::state::{Config, CONFIG, ISMANAGER, WHITELISTEDTOKEN};

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
