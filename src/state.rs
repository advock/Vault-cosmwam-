use cosmwasm_std::{Addr, Int256, Uint128, Uint256};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Position {
    pub size: Uint256,
    pub collateral: Uint256,
    pub averagePrice: Uint256,
    pub entryFundingRate: Uint256,
    pub reserveAmount: Uint256,
    pub realisedPnL: Int256,
    pub lastIncreasedTime: Uint256,
}

pub const ADMIN: Admin = Admin::new("admin");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub bank_addr: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub is_initialized: bool,
    pub is_swap_enabled: bool,
    pub is_leverage_enabled: bool,
    pub vault_utils: Addr,
    pub error_controller: Addr,
    pub router: Addr,
    pub price_feed: Addr,
    pub usdg: Addr,
    pub gov: Addr,
    pub whitelisted_token_count: Uint256,
    pub all_whitelisted_tokens: Vec<Addr>,
    pub max_leverage: Uint256,
    pub liquidation_fee_usd: Uint256,
    pub tax_basis_points: Uint256,
    pub stable_tax_basis_points: Uint256,
    pub mint_burn_fee_basis_points: Uint256,
    pub swap_fee_basis_points: Uint256,
    pub stable_swap_fee_basis_points: Uint256,
    pub margin_fee_basis_points: Uint256,
    pub min_profit_time: Uint256,
    pub has_dynamic_fees: bool,
    pub funding_interval: u128,
    pub funding_rate_factor: u128,
    pub stable_funding_rate_factor: u128,
    pub total_token_weights: Uint256,
    pub include_amm_price: bool,
    pub use_swap_pricing: bool,
    pub in_manager_mode: bool,
    pub in_private_liquidation_mode: bool,
    pub max_gas_price: Uint256,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const ISLIQUIDATOR: Map<Addr, bool> = Map::new("is-liquidator");
pub const ISMANAGER: Map<Addr, bool> = Map::new("is-manager");
pub const WHITELISTEDTOKEN: Map<Addr, bool> = Map::new("white-listed-token");
pub const TOKENDECIMAL: Map<Addr, Uint256> = Map::new("token-decimal");
pub const MINPROFITBASISPOINT: Map<Addr, Uint256> = Map::new("min-profit-basis-Poinut");
pub const STABLETOKEN: Map<Addr, bool> = Map::new("stable-token");
pub const SHORTABLETOKEN: Map<Addr, bool> = Map::new("shortable-token");

pub const TOKENBALANCE: Map<Addr, Uint128> = Map::new("token-balance");
pub const TOKENWEIGHT: Map<Addr, Uint256> = Map::new("tokenWeights");
pub const USDGAMOUNT: Map<Addr, Uint256> = Map::new("usdg-amount");
pub const MAXUSDGAMOUNT: Map<Addr, Uint256> = Map::new("max-USDG-amount");
pub const POOLAMOUNT: Map<Addr, Uint128> = Map::new("pool-amount");
pub const RESERVEDAMOUNTS: Map<Addr, Uint128> = Map::new("reserve - amount");
pub const BUFFERAMOUNT: Map<Addr, Uint256> = Map::new("buffer-amount");
pub const GUARANTEEUSD: Map<Addr, Uint256> = Map::new("guarantee-usdt");
pub const CUMULATIVEFUNDINGRATE: Map<Addr, Uint128> = Map::new("cumulative-funding-rate");
pub const LASTFUNDINTIME: Map<Addr, u128> = Map::new("lastFundingTimes");

pub const POSITION: Map<u128, Position> = Map::new("position");

pub const FEERESERVED: Map<Addr, Uint128> = Map::new("fee-reserved");
pub const GLOBALSHORTSIZE: Map<Addr, Uint256> = Map::new("global-short-size");
pub const GLOBALSHORTAVERAGEPRICE: Map<Addr, Uint256> = Map::new("global-short-average-price");
pub const MAXGLOBALSHORTSIZE: Map<Addr, Uint256> = Map::new("max-global-short-size");
