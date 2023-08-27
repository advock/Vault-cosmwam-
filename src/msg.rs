use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub _router: Addr,
    pub _usdg: Addr,
    pub _priceFeed: Addr,
    pub _liquidationFeeUsd: String,
    pub _fundingRateFactor: String,
    pub _stableFundingRateFactor: String,
    pub _bankAddr: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetVaultUtils {
        _utilisAddr: Addr,
    },

    SetInManagerMode {
        Inmanagermode: bool,
    },

    SetManager {
        ismanager: bool,
        address: Addr,
    },

    SetInPrivateLiqMode {
        InPrivateLiqMode: bool,
    },

    Setliquidator {
        is_active: bool,
        liquidator: Addr,
    },

    SetIsSwapEnabled {
        is_swap_enable: bool,
    },

    SetIsLeverageEnabled {
        is_Leverage_enable: bool,
    },

    SetMaxGasPrice {
        _max_gas_price: Uint128,
    },

    SetGov {
        gov: Addr,
    },

    SetPriceFeed {
        _price_feed: Addr,
    },

    SetMaxLeverage {
        _maxLeverage: Uint128,
    },

    SetBufferAmount {
        _token: Addr,
        _amount: Uint128,
    },

    SetMaxGlobalShortSize {
        _token: Addr,
        _amount: Uint128,
    },

    SetFess {
        _taxBasisPoints: Uint128,
        _stableTaxBasisPoints: Uint128,
        _mintBurnFeeBasisPoints: Uint128,
        _swapFeeBasisPoints: Uint128,
        _stableSwapFeeBasisPoints: Uint128,
        _marginFeeBasisPoints: Uint128,
        _liquidationFeeUsd: Uint128,
        _minProfitTime: Uint128,
        _hasDynamicFees: bool,
    },

    SetFundingRate {
        _fundingInterval: u128,
        _fundingRateFactor: u128,
        _stableFundingRateFactor: u128,
    },

    SetTokenConfig {
        _token: Addr,
        _tokenDecimals: Uint128,
        _tokenWeight: Uint128,
        _minProfitBps: Uint128,
        _maxUsdgAmount: Uint128,
        _isStable: bool,
        _isShortable: bool,
    },

    ClearTokenConfig {
        _token: Addr,
    },

    WithdrawFees {
        _token: Addr,
        _receiver: Addr,
    },

    SetUsdgAmount {
        _token: Addr,
        _amount: Uint128,
    },

    Upgrade {
        _newVault: Addr,
        _token: Addr,
        _amount: Uint128,
    },

    DirectPoolDeposit {
        _token: Addr,
    },

    BuyUSDG {
        _token: Addr,
        _receiver: Addr,
    },

    SellUSDG {
        _token: Addr,
        _receiver: Addr,
    },

    UpdateCumulativeFundingRate {
        _collateralToken: Addr,
        _indexToken: Addr,
    },

    Swap {
        _tokenIn: Addr,
        _tokenOut: Addr,
        _receiver: Addr,
    },

    IncreasePosition {
        _account: Addr,
        _collateralToken: Addr,
        _indexToken: Addr,
        _sizeDelta: Uint128,
        _isLong: bool,
    },

    DecreasePosition {
        _account: Addr,
        _collateralToken: Addr,
        _indexToken: Addr,
        _collateralDelta: Uint128,
        _sizeDelta: Uint128,
        _isLong: bool,
        _receiver: Addr,
    },

    ReduceCollateral {
        _account: Addr,
        _collateralToken: Addr,
        _indexToken: Addr,
        _collateralDelta: Uint128,
        _sizeDelta: Uint128,
        _isLong: bool,
    },

    LiquidatePosition {
        _account: Addr,
        _collateralToken: Addr,
        _indexToken: Addr,
        _isLong: bool,
        _receiver: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
