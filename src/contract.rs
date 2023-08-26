use crate::bank::{get_max_wager, is_asset_whitelisted, pay_in, pay_out};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult, SubMsg, Uint128, Uint256, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use cw_controllers::Admin;

use crate::error::ContractError;
use crate::helpers::{
    _decreaseUsdgAmount, _increasePoolAmount, _increaseUsdgAmount, balance_cw20_tokens,
    transfer_cw20_tokens, updateCumulativeFundingRate, validate,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query::{check_whitelisted_token, query_config};
use crate::state::{
    Config, Position, State, ADMIN, BUFFERAMOUNT, CONFIG, CUMULATIVEFUNDINGRATE, FEERESERVED,
    ISLIQUIDATOR, ISMANAGER, LASTFUNDINTIME, MAXGLOBALSHORTSIZE, MAXUSDGAMOUNT,
    MINPROFITBASISPOINT, POOLAMOUNT, RESERVEDAMOUNTS, SHORTABLETOKEN, STABLETOKEN, STATE,
    TOKENDECIMAL, TOKENWEIGHT, USDGAMOUNT, WHITELISTEDTOKEN,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const UNINITIALIZED_ADDRESS: &str = "UNINITIALIZED";

const BASIS_POINTS_DIVISOR: Uint256 = Uint256::from_u128(10000);
const FUNDING_RATE_PRECISION: Uint256 = Uint256::from_u128(1000000);
const PRICE_PRECISION: u128 = 10u128.pow(30);
const MIN_LEVERAGE: Uint256 = Uint256::from_u128(10000);
const USDG_DECIMALS: Uint256 = Uint256::from_u128(18);
const MAX_FEE_BASIS_POINTS: Uint256 = Uint256::from_u128(500);
const MAX_LIQUIDATION_FEE_USD: Uint256 = Uint256::from_u128(100);
const MIN_FUNDING_RATE_INTERVAL: u128 = 1 * 60 * 60; // 1 hour in seconds
const MAX_FUNDING_RATE_FACTOR: u128 = 10000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State {
        bank_addr: _msg._bankAddr,
    };

    STATE.save(_deps.storage, &state);

    ADMIN.set(_deps, Some((_info.clone().sender)));

    let config = Config {
        is_initialized: true,
        is_swap_enabled: true,
        is_leverage_enabled: true,
        vault_utils: Addr::unchecked(UNINITIALIZED_ADDRESS),
        error_controller: Addr::unchecked(UNINITIALIZED_ADDRESS),
        router: _msg._router,
        price_feed: _msg._priceFeed,
        usdg: _msg._usdg,
        gov: _info.sender,
        whitelisted_token_count: Default::default(),
        all_whitelisted_tokens: Vec::new(),
        max_leverage: Uint256::from(50 * 10000 as u128),
        liquidation_fee_usd: _msg._liquidationFeeUsd,
        tax_basis_points: Uint256::from(50 as u128),
        stable_tax_basis_points: Uint256::from(20 as u128),
        mint_burn_fee_basis_points: Uint256::from(30 as u128),
        swap_fee_basis_points: Uint256::from(30 as u128),
        stable_swap_fee_basis_points: Uint256::from(4 as u128),
        margin_fee_basis_points: Uint256::from(10 as u128),
        min_profit_time: Default::default(),
        has_dynamic_fees: false,
        funding_interval: 8 * 60 * 60,
        funding_rate_factor: _msg._fundingRateFactor,
        stable_funding_rate_factor: _msg._stableFundingRateFactor,
        total_token_weights: Default::default(),
        include_amm_price: true,
        use_swap_pricing: false,
        in_manager_mode: false,
        in_private_liquidation_mode: false,
        max_gas_price: Default::default(),
    };

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

pub fn setVaultUtils(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    _utilisAddr: Addr,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    let mut config = query_config(_deps.as_ref())?;
    config.vault_utils = _utilisAddr;

    CONFIG.save(_deps.storage, &config);

    Ok(Response::new())
}

pub fn setInManagerMode(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    Inmanagermode: bool,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    let mut config = query_config(_deps.as_ref())?;
    config.in_manager_mode = true;

    CONFIG.save(_deps.storage, &config);

    Ok(Response::new())
}

pub fn setManager(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    ismanager: bool,
    address: Addr,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender)?;

    ISMANAGER.save(_deps.storage, address, &ismanager);

    Ok(Response::new())
}

pub fn setInPrivateLiqMode(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    InPrivateLiqMode: bool,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    let mut config = query_config(_deps.as_ref())?;
    config.in_private_liquidation_mode = true;

    CONFIG.save(_deps.storage, &config);

    Ok(Response::new())
}
pub fn setliquidator(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    is_active: bool,
    liquidator: Addr,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender)?;

    ISLIQUIDATOR.save(_deps.storage, liquidator, &is_active);

    Ok(Response::new())
}

pub fn setIsSwapEnabled(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    is_swap_enable: bool,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    let mut config = query_config(_deps.as_ref())?;
    config.is_swap_enabled = true;

    CONFIG.save(_deps.storage, &config);

    Ok(Response::new())
}

pub fn setIsLeverageEnabled(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    is_Leverage_enable: bool,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    let mut config = query_config(_deps.as_ref())?;
    config.is_leverage_enabled = true;

    CONFIG.save(_deps.storage, &config);

    Ok(Response::new())
}

pub fn setMaxGasPrice(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    _max_gas_price: Uint256,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    let mut config = query_config(_deps.as_ref())?;
    config.max_gas_price = _max_gas_price;

    CONFIG.save(_deps.storage, &config);

    Ok(Response::new())
}

pub fn set_gov(
    deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    gov: Addr,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(deps.as_ref(), &_info.sender);
    let mut admin_storage = ADMIN.get(deps.as_ref())?;

    let mut config = query_config(deps.as_ref())?;
    config.gov = gov.clone();

    CONFIG.save(deps.storage, &config);
    ADMIN.set(deps, Some(gov.clone()));

    Ok(Response::new())
}

pub fn setPriceFeed(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    _price_feed: Addr,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    let mut config = query_config(_deps.as_ref())?;
    config.price_feed = _price_feed;

    CONFIG.save(_deps.storage, &config);

    Ok(Response::new())
}

pub fn setMaxLeverage(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    _maxLeverage: Uint256,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    let mut config = query_config(_deps.as_ref())?;
    config.max_gas_price = _maxLeverage;

    CONFIG.save(_deps.storage, &config);

    Ok(Response::new())
}

pub fn setBufferAmount(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    _token: Addr,
    _amount: Uint256,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    BUFFERAMOUNT.save(_deps.storage, _token, &_amount);

    Ok(Response::new())
}

pub fn setMaxGlobalShortSize(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    _token: Addr,
    _amount: Uint256,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender)?;

    MAXGLOBALSHORTSIZE.save(_deps.storage, _token, &_amount);

    Ok(Response::new())
}

pub fn setFess(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    _taxBasisPoints: Uint256,
    _stableTaxBasisPoints: Uint256,
    _mintBurnFeeBasisPoints: Uint256,
    _swapFeeBasisPoints: Uint256,
    _stableSwapFeeBasisPoints: Uint256,
    _marginFeeBasisPoints: Uint256,
    _liquidationFeeUsd: Uint256,
    _minProfitTime: Uint256,
    _hasDynamicFees: bool,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    validate(_taxBasisPoints <= MAX_FEE_BASIS_POINTS, "err");
    validate(_stableTaxBasisPoints <= MAX_FEE_BASIS_POINTS, "err");
    validate(_mintBurnFeeBasisPoints <= MAX_FEE_BASIS_POINTS, "err");
    validate(_swapFeeBasisPoints <= MAX_FEE_BASIS_POINTS, "err");
    validate(_stableSwapFeeBasisPoints <= MAX_FEE_BASIS_POINTS, "err");
    validate(_marginFeeBasisPoints <= MAX_FEE_BASIS_POINTS, "err");
    validate(_liquidationFeeUsd <= MAX_LIQUIDATION_FEE_USD, "err");

    let mut config = query_config(_deps.as_ref())?;

    config.tax_basis_points = _taxBasisPoints;
    config.stable_tax_basis_points = _stableTaxBasisPoints;
    config.mint_burn_fee_basis_points = _mintBurnFeeBasisPoints;
    config.swap_fee_basis_points = _swapFeeBasisPoints;
    config.stable_swap_fee_basis_points = _stableSwapFeeBasisPoints;
    config.margin_fee_basis_points = _marginFeeBasisPoints;
    config.liquidation_fee_usd = _liquidationFeeUsd;
    config.min_profit_time = _minProfitTime;
    config.has_dynamic_fees = _hasDynamicFees;

    CONFIG.save(_deps.storage, &config)?;

    Ok(Response::new())
}

pub fn setFundingRate(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    _fundingInterval: u128,
    _fundingRateFactor: u128,
    _stableFundingRateFactor: u128,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);

    validate(_fundingInterval >= MIN_FUNDING_RATE_INTERVAL, "err");
    validate(_fundingRateFactor <= MAX_FUNDING_RATE_FACTOR, "err");
    validate(_stableFundingRateFactor <= MAX_FUNDING_RATE_FACTOR, "err");

    let mut config = query_config(_deps.as_ref())?;

    config.funding_interval = _fundingInterval;
    config.funding_rate_factor = _fundingRateFactor;
    config.stable_funding_rate_factor = _stableFundingRateFactor;

    CONFIG.save(_deps.storage, &config)?;

    Ok(Response::new())
}

pub fn setTokenConfig(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    _token: Addr,
    _tokenDecimals: Uint256,
    _tokenWeight: Uint256,
    _minProfitBps: Uint256,
    _maxUsdgAmount: Uint256,
    _isStable: bool,
    _isShortable: bool,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);
    let mut config = query_config(_deps.as_ref())?;

    if !check_whitelisted_token(_deps.as_ref(), _token.clone())? {
        config.whitelisted_token_count = config.whitelisted_token_count + Uint256::one();

        config.all_whitelisted_tokens.push(_token.clone());
    }

    let weight = TOKENWEIGHT.load(_deps.storage, _token.clone())?;

    let mut _totalTokenWeights: Uint256 = config.total_token_weights;
    _totalTokenWeights = _totalTokenWeights - weight;

    WHITELISTEDTOKEN.save(_deps.storage, _token.clone(), &true);
    TOKENDECIMAL.save(_deps.storage, _token.clone(), &_tokenDecimals);
    TOKENWEIGHT.save(_deps.storage, _token.clone(), &_tokenWeight);
    MINPROFITBASISPOINT.save(_deps.storage, _token.clone(), &_minProfitBps);
    MAXUSDGAMOUNT.save(_deps.storage, _token.clone(), &_maxUsdgAmount);
    STABLETOKEN.save(_deps.storage, _token.clone(), &_isStable);
    SHORTABLETOKEN.save(_deps.storage, _token.clone(), &_isShortable);

    config.total_token_weights = _totalTokenWeights + _tokenWeight;

    CONFIG.save(_deps.storage, &config)?;

    Ok(Response::new())
}

pub fn clearTokenConfig(
    _deps: DepsMut,
    api: &dyn Api,
    _info: MessageInfo,
    _token: Addr,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender);
    let mut config = query_config(_deps.as_ref())?;

    if check_whitelisted_token(_deps.as_ref(), _token.clone())? {
        config.whitelisted_token_count = config.whitelisted_token_count - Uint256::one();
        let weight = TOKENWEIGHT.load(_deps.storage, _token.clone())?;
        config.total_token_weights = config.total_token_weights - weight;

        CONFIG.save(_deps.storage, &config)?;
    } else {
        return Err(ContractError::Unauthorized {}); // token is not white listed
    }

    WHITELISTEDTOKEN.remove(_deps.storage, _token.clone());
    TOKENDECIMAL.remove(_deps.storage, _token.clone());
    TOKENWEIGHT.remove(_deps.storage, _token.clone());
    MINPROFITBASISPOINT.remove(_deps.storage, _token.clone());
    MAXUSDGAMOUNT.remove(_deps.storage, _token.clone());
    STABLETOKEN.remove(_deps.storage, _token.clone());
    SHORTABLETOKEN.remove(_deps.storage, _token.clone());

    Ok(Response::new())
}

pub fn withdrawFees(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _receiver: Addr,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender)?;

    let amount = FEERESERVED.load(_deps.storage, _token.clone())?;
    if amount == Uint128::zero() {
        return Err(ContractError::Unauthorized {});
    }

    FEERESERVED.save(_deps.storage, _token.clone(), &Uint128::zero());

    let state = STATE.load(_deps.storage)?;

    let mut res = Response::new();

    res = res.add_submessage(pay_out(
        _env.clone(),
        state.bank_addr.clone(),
        0,
        _info.funds[0].denom.clone(),
        _info.sender.clone(),
        amount.u128(),
    ));

    Ok(Response::new())
}

pub fn setUsdgAmount(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _amount: Uint256,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender)?;

    let usdgAmount = USDGAMOUNT.load(_deps.storage, _token.clone())?;

    if _amount > usdgAmount {
        let res = _increaseUsdgAmount(_deps, _env, _info, _token, _amount)?;
        return Ok(res);
    } else {
        let res = _decreaseUsdgAmount(_deps, _env, _info, _token, _amount)?;
        return Ok(res);
    }
}

pub fn upgrade(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _newVault: Addr,
    _token: Addr,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    ADMIN.is_admin(_deps.as_ref(), &_info.sender)?;

    let msg = transfer_cw20_tokens(_token, _env.contract.address, _newVault, _amount)?;

    Ok(Response::new().add_submessage(SubMsg::new(msg)))
}

pub fn directPoolDeposit(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
) -> Result<Response, ContractError> {
    let whitelistedtoken = WHITELISTEDTOKEN.load(_deps.storage, _token.clone())?;
    validate(whitelistedtoken, "err")?;

    let tokenAmount: Uint128 = balance_cw20_tokens(&_deps, _env.clone(), _token.clone())?;

    validate(tokenAmount > Uint128::zero(), "err");
    _increasePoolAmount(_deps, _env.clone(), _info, _token.clone(), tokenAmount)?;

    let event = Event::new("IncreasePoolAmount")
        .add_attribute("token", _token.as_str())
        .add_attribute("amount", tokenAmount.to_string());

    Ok(Response::new().add_event(event))
}

pub fn buyUSDG(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _receiver: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(_deps.storage)?;
    validate(config.in_manager_mode, "err");

    let whitelistedtoken = WHITELISTEDTOKEN.load(_deps.storage, _token.clone())?;
    validate(whitelistedtoken, "err")?;

    config.use_swap_pricing = true;

    CONFIG.save(_deps.storage, &config)?;

    let tokenAmount: Uint128 = balance_cw20_tokens(&_deps, _env, _token)?;
    validate(tokenAmount > Uint128::zero(), "err");

    let should_update = _updateCumulativeFundingRate(_deps, _env, _info, _token, _token)?;
}

pub fn _updateCumulativeFundingRate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _collateralToken: Addr,
    _indexToken: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(_deps.storage)?;
    let fundinginterval = config.funding_interval;
    let should_update = updateCumulativeFundingRate(_deps, _env, _info)?;
    if (!should_update) {
        return Err(ContractError::Unauthorized {});
    }
    let lastFundingTimes = LASTFUNDINTIME.load(_deps.storage, _collateralToken)?;
    let time_stamp: u128;
    if lastFundingTimes == 0 {
        time_stamp = _env.block.time.seconds() as u128;
        LASTFUNDINTIME.save(_deps.storage, _collateralToken, &time_stamp);
        return Ok(Response::new());
    }

    if (lastFundingTimes + fundinginterval) > _env.block.time.seconds() as u128 {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.save(_deps.storage, &config)?;

    let fundingRate = getNextFundingRate(_deps, _env, _info, _collateralToken)?;

    let mut cumulativeFundingRates = CUMULATIVEFUNDINGRATE.load(_deps.storage, _collateralToken)?;
    cumulativeFundingRates = cumulativeFundingRates + fundingRate;
    CUMULATIVEFUNDINGRATE.save(_deps.storage, _collateralToken, &cumulativeFundingRates);

    let mut lastFundingTimes = LASTFUNDINTIME.load(_deps.storage, _collateralToken)?;
    lastFundingTimes = _env.block.time.seconds() as u128;
    LASTFUNDINTIME.save(_deps.storage, _collateralToken, &lastFundingTimes);

    let event = Event::new("IncreasePoolAmount")
        .add_attribute("token", _collateralToken.as_str())
        .add_attribute("amount", cumulativeFundingRates.to_string());

    Ok(Response::new().add_event(event))
}

pub fn getNextFundingRate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _collateralToken: Addr,
) -> Result<Uint128, ContractError> {
    let lastFundingTimes = LASTFUNDINTIME.load(_deps.storage, _collateralToken)?;
    let config = CONFIG.load(_deps.storage)?;
    let fundinginterval = config.funding_interval;

    if lastFundingTimes + fundinginterval > _env.block.time.seconds() as u128 {
        return Ok(Uint128::zero());
    }

    let intervals: u128 =
        ((_env.block.time.seconds() as u128) - lastFundingTimes) / fundinginterval;
    let poolAmount = POOLAMOUNT.load(_deps.storage, _collateralToken)?;
    if poolAmount == Uint128::zero() {
        return Ok(Uint128::zero());
    }

    let _fundingRateFactor: u128;

    let stableToken = STABLETOKEN.load(_deps.storage, _collateralToken)?;

    if stableToken {
        _fundingRateFactor = config.stable_funding_rate_factor;
    } else {
        _fundingRateFactor = config.funding_rate_factor
    }
    let reserve_amount = RESERVEDAMOUNTS.load(_deps.storage, _collateralToken)?;

    let rate = (_fundingRateFactor * (reserve_amount.u128()) * intervals);

    Ok(Uint128::new(rate) / poolAmount)
}

#[cfg(test)]
mod tests {}
