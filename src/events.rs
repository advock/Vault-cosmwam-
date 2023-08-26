use cosmwasm_std::{Addr, Response, Uint256};

pub struct BuyUSDGEvent {
    account: Addr,
    token: Addr,
    token_amount: Uint256,
    usdg_amount: Uint256,
    fee_basis_points: Uint256,
}

pub struct SellUSDGEvent {
    account: Addr,
    token: Addr,
    usdg_amount: Uint256,
    token_amount: Uint256,
    fee_basis_points: Uint256,
}

pub struct SwapEvent {
    account: Addr,
    token_in: Addr,
    token_out: Addr,
    amount_in: Uint256,
    amount_out: Uint256,
    amount_out_after_fees: Uint256,
    fee_basis_points: Uint256,
}

pub struct IncreasePositionEvent {
    key: [u8; 32],
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    collateral_delta: Uint256,
    size_delta: Uint256,
    is_long: bool,
    price: Uint256,
    fee: Uint256,
}

pub struct DecreasePositionEvent {
    key: [u8; 32],
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    collateral_delta: Uint256,
    size_delta: Uint256,
    is_long: bool,
    price: Uint256,
    fee: Uint256,
}

pub struct LiquidatePositionEvent {
    key: [u8; 32],
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
    size: Uint256,
    collateral: Uint256,
    reserve_amount: Uint256,
    realised_pnl: i256,
    mark_price: Uint256,
}

pub struct UpdatePositionEvent {
    key: [u8; 32],
    size: Uint256,
    collateral: Uint256,
    average_price: Uint256,
    entry_funding_rate: Uint256,
    reserve_amount: Uint256,
    realised_pnl: i256,
    mark_price: Uint256,
}

pub struct ClosePositionEvent {
    key: [u8; 32],
    size: Uint256,
    collateral: Uint256,
    average_price: Uint256,
    entry_funding_rate: Uint256,
    reserve_amount: Uint256,
    realised_pnl: i256,
}

pub struct UpdateFundingRate {
    token: Addr,
    funding: Uint256,
}
pub struct CollectSwapFees {
    token: Addr,
    feeUSD: Uint256,
    feetoken: Uint256,
}

pub struct CollectMarginFees {
    token: Addr,
    feeUSD: Uint256,
    feetoken: Uint256,
}

pub struct DirectPoolDeposit {
    token: Addr,
    amount: Uint256,
}

pub struct IncreasePoolAmount {
    token: Addr,
    amount: Uint256,
}

pub struct DecreasePoolAmount {
    token: Addr,
    amount: Uint256,
}

pub struct IncreaseUsdgAmount {
    token: Addr,
    amount: Uint256,
}

pub struct DecreaseUsdgAmount {
    token: Addr,
    amount: Uint256,
}

pub struct IncreaseReservedAmount {
    token: Addr,
    amount: Uint256,
}

pub struct DecreaseReservedAmount {
    token: Addr,
    amount: Uint256,
}

pub struct IncreaseGuaranteedUsd {
    token: Addr,
    amount: Uint256,
}

pub struct DecreaseGuaranteedUsd {
    token: Addr,
    amount: Uint256,
}
