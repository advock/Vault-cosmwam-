use std::str::FromStr;

use crate::bank::{get_max_wager, is_asset_whitelisted, pay_in, pay_out};
use crate::events::{DecreasePositionEvent, DecreaseReservedAmount};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, Event, Int128, MessageInfo,
    Response, StdResult, SubMsg, Uint128, Uint256, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use cw_controllers::Admin;

use crate::error::ContractError;
use crate::helpers::{
    _collect_margin_fees, _decreaseGlobalShortSize, _decreaseGuaranteedUsd, _decreasePoolAmount,
    _decreaseReservedAmount, _decreaseUsdgAmount, _increaseGuaranteedUsd, _increasePoolAmount,
    _increaseReservedAmount, _increaseUsdgAmount, _validateTokens, balance_cw20_tokens,
    getBuyUsdgFeeBasisPoints, getSellUsdgFeeBasisPoints, getSwapFeeBasisPoints, get_delta,
    get_entry_funding_rate, get_max_price, get_min_price, get_next_average_price,
    get_next_global_short_average_price, token_to_usd_min, transfer_cw20_tokens,
    updateCumulativeFundingRate, update_token_bal, usdToTokenMax, usd_to_token_min,
    validLiquidation, validate,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query::{check_whitelisted_token, get_position, get_position_key, query_config};
use crate::state::{
    Config, Position, State, ADMIN, BUFFERAMOUNT, CONFIG, CUMULATIVEFUNDINGRATE, FEERESERVED,
    GLOBALSHORTAVERAGEPRICE, GLOBALSHORTSIZE, ISLIQUIDATOR, ISMANAGER, LASTFUNDINTIME,
    MAXGLOBALSHORTSIZE, MAXUSDGAMOUNT, MINPROFITBASISPOINT, POOLAMOUNT, POSITION, RESERVEDAMOUNTS,
    SHORTABLETOKEN, STABLETOKEN, STATE, TOKENDECIMAL, TOKENWEIGHT, USDGAMOUNT, WHITELISTEDTOKEN,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const UNINITIALIZED_ADDRESS: &str = "UNINITIALIZED";

pub const BASIS_POINTS_DIVISOR: Uint128 = Uint128::new(10000);
const FUNDING_RATE_PRECISION: Uint128 = Uint128::new(1000000);
const PRICE_PRECISION: Uint128 = Uint128::new(10u128.pow(30));
const MIN_LEVERAGE: Uint128 = Uint128::new(10000);
const USDG_DECIMALS: Uint128 = Uint128::new(6);
const MAX_FEE_BASIS_POINTS: Uint128 = Uint128::new(500);
const MAX_LIQUIDATION_FEE_USD: Uint128 = Uint128::new(100);
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
    _taxBasisPoints: Uint128,
    _stableTaxBasisPoints: Uint128,
    _mintBurnFeeBasisPoints: Uint128,
    _swapFeeBasisPoints: Uint128,
    _stableSwapFeeBasisPoints: Uint128,
    _marginFeeBasisPoints: Uint128,
    _liquidationFeeUsd: Uint128,
    _minProfitTime: Uint128,
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
    _tokenDecimals: Uint128,
    _tokenWeight: Uint256,
    _minProfitBps: Uint256,
    _maxUsdgAmount: Uint128,
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

    let mut _totalTokenWeights: Uint128 = config.total_token_weights;
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
    _amount: Uint128,
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
    mut _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _receiver: Addr,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(_deps.storage)?;
    validate(config.in_manager_mode, "err");

    let whitelistedtoken = WHITELISTEDTOKEN.load(_deps.storage, _token.clone())?;
    validate(whitelistedtoken, "err")?;

    config.use_swap_pricing = true;

    CONFIG.save(_deps.storage, &config)?;

    let tokenAmount: Uint128 = balance_cw20_tokens(&_deps.branch(), _env.clone(), _token.clone())?;
    validate(tokenAmount > Uint128::zero(), "err");

    let should_update = _updateCumulativeFundingRate(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        _token.clone(),
    )?;

    let price: Uint128 =
        get_min_price(_deps.branch(), _env.clone(), _info.clone(), _token.clone())?;

    let mut usdgAmount: Uint128 = (tokenAmount * price) / PRICE_PRECISION;

    usdgAmount = adjust_decimal(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        config.usdg.clone(),
        usdgAmount,
    )?;

    validate(usdgAmount > Uint128::zero(), "err");

    let feeBasisPoints: Uint128 = getBuyUsdgFeeBasisPoints(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        usdgAmount,
    )?;

    let amountAfterFees: Uint128 = collect_fees(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        tokenAmount,
        feeBasisPoints,
    )?;

    let mut mintAmount: Uint128 = (amountAfterFees * price) / PRICE_PRECISION;
    mintAmount = adjust_decimal(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        config.usdg.clone(),
        mintAmount,
    )?;

    _increaseUsdgAmount(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        mintAmount,
    )?;
    _increasePoolAmount(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        amountAfterFees,
    )?;
    let attributes = Event::new("BuyUSDG")
        .add_attribute("action", "BuyUSDG")
        .add_attribute("receiver", _receiver.as_str())
        .add_attribute("token", _token.as_str())
        .add_attribute("token_amount", usdgAmount.to_string())
        .add_attribute("mint_amount", mintAmount.to_string())
        .add_attribute("fee_basis_points", feeBasisPoints.to_string());

    Ok(Response::new().add_event(attributes))
}

pub fn sellUSDG(
    mut _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _receiver: Addr,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(_deps.storage)?;
    validate(config.in_manager_mode, "err");

    let whitelistedtoken = WHITELISTEDTOKEN.load(_deps.storage, _token.clone())?;
    validate(whitelistedtoken, "err")?;

    config.use_swap_pricing = true;

    CONFIG.save(_deps.storage, &config)?;

    let usdgAmount: Uint128 =
        balance_cw20_tokens(&_deps.branch(), _env.clone(), config.usdg.clone())?;
    validate(usdgAmount > Uint128::zero(), "err");

    let should_update = _updateCumulativeFundingRate(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        _token.clone(),
    )?;

    let redemptionAmount: Uint128 = getRedemptionAmount(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        usdgAmount,
    )?;
    validate(redemptionAmount > Uint128::zero(), "errr");

    _decreaseUsdgAmount(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        usdgAmount,
    )?;
    _decreasePoolAmount(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        redemptionAmount,
    )?;

    update_token_bal(_deps.branch(), _env.clone(), _info.clone(), config.usdg)?;

    let feeBasisPoints: Uint128 = getSellUsdgFeeBasisPoints(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        usdgAmount,
    )?;

    let amountAfterFees: Uint128 = collect_fees(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _token.clone(),
        redemptionAmount,
        feeBasisPoints,
    )?;

    validate(amountAfterFees > Uint128::zero(), "err");
    let mut res = Response::new();

    let state = STATE.load(_deps.storage)?;

    res = res.add_submessage(pay_out(
        _env.clone(),
        state.bank_addr.clone(),
        0,
        _token.clone().into_string(),
        _receiver.clone(),
        amountAfterFees.u128(),
    ));

    let attributes = Event::new("BuyUSDG")
        .add_attribute("action", "BuyUSDG")
        .add_attribute("receiver", _receiver.as_str())
        .add_attribute("token", _token.as_str())
        .add_attribute("token_amount", usdgAmount.to_string())
        .add_attribute("burn_amount", amountAfterFees.to_string())
        .add_attribute("fee_basis_points", feeBasisPoints.to_string());

    Ok(Response::new().add_event(attributes))
}

pub fn _updateCumulativeFundingRate(
    mut _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _collateralToken: Addr,
    _indexToken: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(_deps.storage)?;
    let fundinginterval = config.funding_interval;
    let should_update = updateCumulativeFundingRate(_deps.branch(), _env.clone(), _info.clone())?;
    if (!should_update) {
        return Err(ContractError::Unauthorized {});
    }
    let lastFundingTimes = LASTFUNDINTIME.load(_deps.storage, _collateralToken.clone())?;
    let time_stamp: u128;
    if lastFundingTimes == 0 {
        time_stamp = _env.clone().block.time.seconds() as u128;
        LASTFUNDINTIME.save(
            _deps.branch().storage,
            _collateralToken.clone(),
            &time_stamp,
        );
        return Ok(Response::new());
    }

    if (lastFundingTimes + fundinginterval) > _env.clone().block.time.seconds() as u128 {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.save(_deps.branch().storage, &config)?;

    let fundingRate = getNextFundingRate(
        _deps.branch(),
        _env.clone(),
        _info,
        _collateralToken.clone(),
    )?;

    let mut cumulativeFundingRates =
        CUMULATIVEFUNDINGRATE.load(_deps.storage, _collateralToken.clone())?;
    cumulativeFundingRates = cumulativeFundingRates + fundingRate;
    CUMULATIVEFUNDINGRATE.save(
        _deps.branch().storage,
        _collateralToken.clone(),
        &cumulativeFundingRates,
    );

    let mut lastFundingTimes =
        LASTFUNDINTIME.load(_deps.branch().storage, _collateralToken.clone())?;
    lastFundingTimes = _env.clone().block.time.seconds() as u128;
    LASTFUNDINTIME.save(
        _deps.branch().storage,
        _collateralToken.clone(),
        &lastFundingTimes,
    );

    let event = Event::new("IncreasePoolAmount")
        .add_attribute("token", _collateralToken.clone().as_str())
        .add_attribute("amount", cumulativeFundingRates.to_string());

    Ok(Response::new().add_event(event))
}

pub fn getNextFundingRate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _collateralToken: Addr,
) -> Result<Uint128, ContractError> {
    let lastFundingTimes = LASTFUNDINTIME.load(_deps.storage, _collateralToken.clone())?;
    let config = CONFIG.load(_deps.storage)?;
    let fundinginterval = config.funding_interval;

    if lastFundingTimes + fundinginterval > _env.block.time.seconds() as u128 {
        return Ok(Uint128::zero());
    }

    let intervals: u128 =
        ((_env.block.time.seconds() as u128) - lastFundingTimes) / fundinginterval;
    let poolAmount = POOLAMOUNT.load(_deps.storage, _collateralToken.clone())?;
    if poolAmount == Uint128::zero() {
        return Ok(Uint128::zero());
    }

    let _fundingRateFactor: u128;

    let stableToken = STABLETOKEN.load(_deps.storage, _collateralToken.clone())?;

    if stableToken {
        _fundingRateFactor = config.stable_funding_rate_factor;
    } else {
        _fundingRateFactor = config.funding_rate_factor
    }
    let reserve_amount = RESERVEDAMOUNTS.load(_deps.storage, _collateralToken.clone())?;

    let rate = (_fundingRateFactor * (reserve_amount.u128()) * intervals);

    Ok(Uint128::new(rate) / poolAmount)
}

pub fn adjust_decimal(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _tokenDev: Addr,
    _tokenMul: Addr,
    _amount: Uint128,
) -> Result<Uint128, ContractError> {
    let decimalsDiv: Uint128;
    let config = CONFIG.load(_deps.storage)?;
    if _tokenDev == config.usdg {
        decimalsDiv = USDG_DECIMALS
    } else {
        decimalsDiv = TOKENDECIMAL.load(_deps.storage, _tokenDev)?;
    }

    let decimalsMul: Uint128;

    if _tokenMul == config.usdg {
        decimalsMul = USDG_DECIMALS
    } else {
        decimalsMul = TOKENDECIMAL.load(_deps.storage, _tokenMul)?;
    }

    let res: Uint128 = (_amount * Uint128::new(10).pow(decimalsMul.u128() as u32))
        / Uint128::new(10).pow(decimalsDiv.u128() as u32);

    Ok(res)
}
pub fn collect_fees(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _amount: Uint128,
    _fee_basis_points: Uint128,
) -> Result<Uint128, ContractError> {
    let afterFeeAmount: Uint128 =
        _amount * (BASIS_POINTS_DIVISOR - _fee_basis_points) / BASIS_POINTS_DIVISOR;
    let feeAmount: Uint128 = _amount - afterFeeAmount;

    let feeReserves = FEERESERVED.load(_deps.storage, _token)?;

    Ok((feeReserves))
}

pub fn getRedemptionAmount(
    mut _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: Addr,
    _usdgAmount: Uint128,
) -> Result<Uint128, ContractError> {
    let price: Uint128 =
        get_max_price(_deps.branch(), _env.clone(), _info.clone(), _token.clone())?;
    let redemptionAmount: Uint128 = (_usdgAmount * PRICE_PRECISION) / price;
    let config = CONFIG.load(_deps.storage)?;

    let res = adjust_decimal(
        _deps,
        _env.clone(),
        _info.clone(),
        config.usdg,
        _token.clone(),
        redemptionAmount,
    )?;
    Ok(res)
}

pub fn swap(
    mut _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _tokenIn: Addr,
    _tokenOut: Addr,
    _receiver: Addr,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(_deps.storage)?;
    let whitelistedTokensIn = WHITELISTEDTOKEN.load(_deps.storage, _tokenIn.clone())?;
    let whitelistedTokensOut = WHITELISTEDTOKEN.load(_deps.storage, _tokenOut.clone())?;

    validate(config.is_swap_enabled, "err")?;
    validate(whitelistedTokensIn, "err")?;
    validate(whitelistedTokensOut, "err")?;
    validate(_tokenIn != _tokenOut, "err")?;

    config.use_swap_pricing = true;
    _updateCumulativeFundingRate(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _tokenIn.clone(),
        _tokenIn.clone(),
    )?;
    _updateCumulativeFundingRate(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _tokenOut.clone(),
        _tokenOut.clone(),
    )?;

    let amountIn: Uint128 = balance_cw20_tokens(&_deps.branch(), _env.clone(), _tokenIn.clone())?;
    validate(amountIn > Uint128::zero(), "err")?;

    let priceIn = get_min_price(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _tokenIn.clone(),
    )?;
    let priceOut = get_max_price(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _tokenOut.clone(),
    )?;

    let mut amountOut: Uint128 = amountIn * priceIn / priceOut;

    amountOut = adjust_decimal(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _tokenIn.clone(),
        _tokenOut.clone(),
        amountOut,
    )?;

    let mut usdgAmount: Uint128 = amountIn * priceIn / PRICE_PRECISION;
    usdgAmount = adjust_decimal(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _tokenIn.clone(),
        config.clone().usdg,
        usdgAmount,
    )?;

    let feeBasisPoints: Uint128 = getSwapFeeBasisPoints(
        _deps.branch(),
        _env.clone(),
        _info.clone(),
        _tokenIn.clone(),
        _tokenOut.clone(),
        usdgAmount,
    )?;

    CONFIG.save(_deps.storage, &config)?;
    Ok(Response::new())
}

pub fn increasePosition(
    mut _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _account: Addr,
    _collateralToken: Addr,
    _indexToken: Addr,
    _sizeDelta: Uint128,
    _isLong: bool,
) -> Result<Response, ContractError> {
    let cofig = CONFIG.load(_deps.storage)?;

    validate(cofig.is_leverage_enabled, "err")?;
    _validateTokens(&_deps, _collateralToken, _indexToken, _isLong)?;

    _updateCumulativeFundingRate(_deps, _env, _info, _collateralToken, _indexToken)?;
    let key = get_position_key(_account, _collateralToken, _indexToken, _isLong);

    let price: Uint128;

    if _isLong {
        price = get_max_price(_deps, _env, _info, _indexToken)?;
    } else {
        price = get_min_price(_deps, _env, _info, _indexToken)?;
    }

    let mut position = get_position(_deps.as_ref(), key)?;

    if position.size == Uint128::zero() {
        position.averagePrice = price;
    }
    if (position.size > Uint128::zero() && _sizeDelta > Uint128::zero()) {
        position.averagePrice = get_next_average_price(
            _deps,
            _env,
            _info,
            &_indexToken,
            position.size,
            position.averagePrice,
            _isLong,
            price,
            _sizeDelta,
            position.lastIncreasedTime,
        )?;
    }

    let fess = _collect_margin_fees(
        _deps,
        _env,
        _info,
        _account,
        _collateralToken,
        _indexToken,
        _isLong,
        _sizeDelta.u128(),
        position.size.u128(),
        position.entryFundingRate.u128(),
    )?
    .attributes[0]
        .value;

    let _fees = Uint128::from_str(&fess)?;

    let mut hasProfit: bool;
    let mut adjustedDelta: Uint128;

    let collateralDelta = balance_cw20_tokens(&_deps, _env, _collateralToken)?;
    let collateralDeltaUsd =
        token_to_usd_min(_deps, _env, _info, _collateralToken, collateralDelta.u128())?;

    position.collateral = position.collateral + collateralDeltaUsd;
    position.collateral = position.collateral - _fees;

    position.entryFundingRate =
        get_entry_funding_rate(_deps, _collateralToken, _indexToken, _isLong)?;

    position.size = position.size + _sizeDelta;
    position.lastIncreasedTime = _env.block.time.seconds();
    validate(position.size > Uint128::zero(), "err");

    let reserveDelta = usdToTokenMax(_deps, _env, _info, _collateralToken, _sizeDelta)?;
    position.reserveAmount = position.reserveAmount + reserveDelta;
    _increaseReservedAmount(_deps, _collateralToken, reserveDelta);

    if _isLong {
        _increaseGuaranteedUsd(_deps, _collateralToken, _sizeDelta + _fees);
        _decreaseGuaranteedUsd(_deps, _collateralToken, collateralDeltaUsd);
        _increasePoolAmount(_deps, _env, _info, _collateralToken, collateralDelta);
        _decreasePoolAmount(
            _deps,
            _env,
            _info,
            _collateralToken,
            Uint128::new(usd_to_token_min(
                _deps,
                _env,
                _info,
                _collateralToken,
                _fees.u128(),
            )?),
        );
    } else {
        let globalShortSizes = GLOBALSHORTSIZE.load(_deps.storage, _indexToken)?;
        if (globalShortSizes == Uint128::zero()) {
            GLOBALSHORTSIZE.save(_deps.storage, _indexToken, &price);
        } else {
            let mut globalShortAveragePrices =
                get_next_global_short_average_price(_deps, _indexToken, price, _sizeDelta)?;
            GLOBALSHORTAVERAGEPRICE.save(_deps.storage, _indexToken, &globalShortAveragePrices)?;
        }
    }

    POSITION.save(_deps.storage, key, &position)?;

    Ok(Response::new())
}

pub fn decreasePosition(
    mut _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _account: Addr,
    _collateralToken: Addr,
    _indexToken: Addr,
    _collateralDelta: Uint128,
    _sizeDelta: Uint128,
    _isLong: bool,
    _receiver: Addr,
) -> Result<Response, ContractError> {
    _updateCumulativeFundingRate(_deps, _env, _info, _collateralToken, _indexToken)?;
    let key = get_position_key(_account, _collateralToken, _indexToken, _isLong);
    let mut position = get_position(_deps.as_ref(), key)?;
    validate(position.size > Uint128::zero(), "err");
    validate(position.size >= _sizeDelta, "err");
    validate(position.collateral >= _collateralDelta, "err");
    let mut config = CONFIG.load(_deps.storage)?;

    let collateral: Uint128 = position.collateral;
    let reserveDelta: Uint128 = position.reserveAmount * _sizeDelta / position.size;
    position.reserveAmount = position.reserveAmount - reserveDelta;
    _decreaseReservedAmount(_deps, _collateralToken, reserveDelta);

    let usdtout: Uint128;

    usdtout = _reduceCollateral(
        _deps,
        _env,
        _info,
        _account,
        _collateralToken,
        _indexToken,
        _collateralDelta,
        _sizeDelta,
        _isLong,
    )?;

    let price: Uint128;

    if position.size != _sizeDelta {
        // Update entry funding rate
        let entry_funding_rate =
            get_entry_funding_rate(_deps, _collateralToken, _indexToken, _isLong)?;

        // Update position size and validate
        position.size = position.size.wrapping_sub(_sizeDelta);

        if _isLong {
            _increaseGuaranteedUsd(_deps, _collateralToken, collateral - position.collateral)?;
        }

        if _isLong {
            price = get_min_price(_deps, _env, _info, _indexToken)?;
        } else {
            price = get_min_price(_deps, _env, _info, _indexToken)?;
        };
    };

    if (!_isLong) {
        (_indexToken, _sizeDelta);
        _decreaseGlobalShortSize(_deps, _indexToken, position.size);
    }
    if usdtout > Uint128::zero() {
        if (_isLong) {
            _decreasePoolAmount(
                _deps,
                _env,
                _info,
                _collateralToken,
                Uint128::new(usd_to_token_min(
                    _deps,
                    _env,
                    _info,
                    _collateralToken,
                    config.liquidation_fee_usd.u128(),
                )?),
            )?;

            let amount = Uint128::new(usd_to_token_min(
                _deps,
                _env,
                _info,
                _collateralToken,
                config.liquidation_fee_usd.u128(),
            )?);
            transfer_cw20_tokens(_collateralToken, _env.contract.address, _receiver, amount);
            return Ok(Response::new().add_attribute("amountafterfees", amount.to_string()));
        }
    }
    POSITION.save(_deps.storage, key, &position);
    let decrease_event = Event::new("decrease_position")
        .add_attribute("account", _account.clone())
        .add_attribute("collateral_token", _collateralToken.clone())
        .add_attribute("index_token", _indexToken.clone())
        .add_attribute("collateral_delta", _collateralDelta.to_string())
        .add_attribute("size_delta", _sizeDelta.to_string())
        .add_attribute("is_long", _isLong.to_string())
        .add_attribute("price", price.to_string())
        .add_attribute("usd_out_after_fee", (usdtout.to_string()));

    Ok(Response::new().add_event(decrease_event))
}

pub fn _reduceCollateral(
    mut _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _account: Addr,
    _collateralToken: Addr,
    _indexToken: Addr,
    _collateralDelta: Uint128,
    _sizeDelta: Uint128,
    _isLong: bool,
) -> Result<Uint128, ContractError> {
    let key = get_position_key(_account, _collateralToken, _indexToken, _isLong);
    let mut position = get_position(_deps.as_ref(), key)?;

    let fess = _collect_margin_fees(
        _deps,
        _env,
        _info,
        _account,
        _collateralToken,
        _indexToken,
        _isLong,
        _sizeDelta.u128(),
        position.size.u128(),
        position.entryFundingRate.u128(),
    )?
    .attributes[0]
        .value;

    let _fees = Uint128::from_str(&fess)?;

    let mut hasProfit: bool;
    let mut adjustedDelta: Uint128;

    let (_hasProfit, delta) = get_delta(
        _deps,
        _env,
        _info,
        _indexToken,
        position.size,
        position.averagePrice,
        _isLong,
        position.lastIncreasedTime,
    )?;
    hasProfit = _hasProfit;

    adjustedDelta = _sizeDelta * delta / position.size;

    let usdtOut: Uint128;

    if (hasProfit && adjustedDelta > Uint128::zero()) {
        position.collateral = position.collateral - adjustedDelta;

        if _isLong {
            let tokenAmount =
                usd_to_token_min(_deps, _env, _info, _collateralToken, adjustedDelta.u128())?;
            _increasePoolAmount(
                _deps,
                _env,
                _info,
                _collateralToken,
                Uint128::new(tokenAmount),
            )?;
        }
        position.realisedPnL = position.realisedPnL - Int128::new(adjustedDelta.u128() as i128);

        if _collateralDelta > Uint128::zero() {
            usdtOut = usdtOut + _collateralDelta;
            position.collateral = position.collateral - _collateralDelta;
        }

        if position.size == _sizeDelta {
            usdtOut = usdtOut + position.collateral;
            position.collateral = Uint128::zero();
        }
    }
    let mut attributes = vec![
        attr("action", "update_pnl"),
        attr("has_profit", hasProfit.to_string()),
        attr("adjusted_delta", adjustedDelta.to_string()),
    ];

    let event = Event::new("update_pnl");

    Ok(usdtOut)
}

pub fn liquidatePosition(
    mut _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _account: Addr,
    _collateralToken: Addr,
    _indexToken: Addr,
    _isLong: bool,
    _receiver: Addr,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(_deps.storage)?;
    if config.in_private_liquidation_mode {
        let isliq = ISLIQUIDATOR.load(_deps.storage, _info.sender)?;
        validate(isliq, "error_message");
    }

    config.include_amm_price = false;
    _updateCumulativeFundingRate(_deps, _env, _info, _collateralToken, _indexToken)?;

    let key = get_position_key(_account, _collateralToken, _indexToken, _isLong);

    let mut position = get_position(_deps.as_ref(), key)?;
    validate(position.size > Uint128::zero(), "error_message");

    let liquidationState: Uint128;
    let marginFees: Uint128;
    (liquidationState, marginFees) = validLiquidation(
        _deps,
        _env,
        _info,
        _account,
        _collateralToken,
        _indexToken,
        _isLong,
        _receiver,
    )?;

    validate(liquidationState != Uint128::zero(), "errr");

    if liquidationState == Uint128::new(2) {
        decreasePosition(
            _deps,
            _env,
            _info,
            _account,
            _collateralToken,
            _indexToken,
            Uint128::zero(),
            position.size,
            _isLong,
            _account,
        );

        config.include_amm_price = true;
        return Ok(Response::new());
    }

    let feeTokens = usd_to_token_min(_deps, _env, _info, _collateralToken, marginFees.u128())?;
    let feeReserves = FEERESERVED.load(_deps.storage, _collateralToken)?;
    FEERESERVED.save(
        _deps.storage,
        _collateralToken,
        &(feeReserves + Uint128::new(feeTokens)),
    )?;
    let event = Event::new("collect_margin_fees")
        .add_attribute("collateral_token", _collateralToken.to_string())
        .add_attribute("margin_fees", marginFees.to_string())
        .add_attribute("fee_tokens", feeTokens.to_string());

    _decreaseReservedAmount(_deps, _collateralToken, position.reserveAmount)?;

    if _isLong {
        _decreaseGuaranteedUsd(_deps, _collateralToken, position.size - position.collateral);
        _decreasePoolAmount(
            _deps,
            _env,
            _info,
            _collateralToken,
            Uint128::new(usd_to_token_min(
                _deps,
                _env,
                _info,
                _collateralToken,
                marginFees.u128(),
            )?),
        )?;
    }

    let markPrice: Uint128;
    if _isLong {
        markPrice = get_min_price(_deps, _env, _info, _indexToken)?;
    } else {
        markPrice = get_max_price(_deps, _env, _info, _indexToken)?;
    }
    let event = Event::new("collect_margin_fees")
        .add_attribute("collateral_token", _collateralToken)
        .add_attribute("margin_fees", marginFees.to_string())
        .add_attribute("fee_tokens", feeTokens.to_string());

    if !_isLong && marginFees < position.collateral {
        let remaining_collateral = position.collateral.checked_sub(marginFees).unwrap();

        let usd_to_token_min = usd_to_token_min(
            _deps,
            _env,
            _info,
            _collateralToken,
            remaining_collateral.u128(),
        )?;

        _increasePoolAmount(
            _deps,
            _env,
            _info,
            _collateralToken,
            Uint128::new(usd_to_token_min),
        )?;
    }

    if _isLong {
        _decreaseGlobalShortSize(_deps, _indexToken, position.size);
    }

    let amount = usd_to_token_min(
        _deps,
        _env,
        _info,
        _collateralToken,
        config.liquidation_fee_usd.u128(),
    )?;

    _decreasePoolAmount(_deps, _env, _info, _collateralToken, Uint128::new(amount));

    transfer_cw20_tokens(
        _collateralToken,
        _env.contract.address,
        _receiver,
        Uint128::new(amount),
    )?;

    config.include_amm_price = true;

    CONFIG.save(_deps.storage, &config);
    POSITION.save(_deps.storage, key, &position);

    Ok(Response::new().add_event(event))
}

pub fn get_Utilisationsition_key(_deps: DepsMut, _token: Addr) -> Result<Uint128, ContractError> {
    let poolAmount = POOLAMOUNT.load(_deps.storage, _token)?;

    if poolAmount > Uint128::zero() {
        return Ok(Uint128::zero());
    }
    let reservedAmounts = RESERVEDAMOUNTS.load(_deps.storage, _token)?;
    let res: Uint128 = reservedAmounts * FUNDING_RATE_PRECISION / poolAmount;

    Ok(res)
}

#[cfg(test)]
mod tests {}
