use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub _router: Addr,
    pub _usdg: Addr,
    pub _priceFeed: Addr,
    pub _liquidationFeeUsd: Uint128,
    pub _fundingRateFactor: u128,
    pub _stableFundingRateFactor: u128,
    pub _bankAddr: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
