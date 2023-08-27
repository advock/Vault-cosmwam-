use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Deps, Env, QueryRequest, SubMsg, Uint128, WasmMsg,
    WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Eq)]
#[serde(rename_all = "snake_case")]

pub enum ExecuteBankMsg {
    PayIn {
        sender: Addr,
        escrowed_amount: u128,
        escrowed_asset: String,
    },
    PayOut {
        sender: Addr,
        escrowed_amount: u128,
        escrowed_asset: String,
        recipient: Addr,
        total_amount: u128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Eq)]
#[serde(rename_all = "snake_case")]

pub enum QueryBankMsg {
    GetMaxWager { asset: String },
    IsAssetWhitelisted { asset: String },
}

pub fn pay_in(env: Env, bank_address: Addr, pay_in_amount: u128, pay_in_denom: String) -> SubMsg {
    let expected = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: bank_address.to_string(),
        funds: vec![Coin {
            denom: pay_in_denom.clone(),
            amount: Uint128::from(pay_in_amount),
        }],
        msg: to_binary(&ExecuteBankMsg::PayIn {
            sender: env.contract.address,
            escrowed_amount: pay_in_amount,
            escrowed_asset: pay_in_denom,
        })
        .unwrap(),
    }));
    expected
}

pub fn pay_out(
    env: Env,
    bank_address: Addr,
    pay_in_amount: u128,
    pay_in_denom: String,
    recipient: Addr,
    total_amount: u128,
) -> SubMsg {
    let expected = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: bank_address.to_string(),
        funds: vec![Coin {
            denom: pay_in_denom.clone(),
            amount: Uint128::from(pay_in_amount),
        }],
        msg: to_binary(&ExecuteBankMsg::PayOut {
            sender: env.contract.address,
            escrowed_amount: pay_in_amount,
            escrowed_asset: pay_in_denom,
            recipient: recipient,
            total_amount: total_amount,
        })
        .unwrap(),
    }));
    expected
}

///// add to response of tx like Response.new().add_submessage(submsg)
///
///Get Contract query response
pub fn get_max_wager(deps: Deps, bank_address: Addr, asset: String) -> Uint128 {
    let query_msg = QueryBankMsg::GetMaxWager { asset: asset };
    let query_response: Uint128 = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: bank_address.to_string(),
            msg: to_binary(&query_msg).unwrap(),
        }))
        .unwrap();

    query_response
}

pub fn is_asset_whitelisted(deps: Deps, bank_address: Addr, asset: String) -> bool {
    let query_msg = QueryBankMsg::IsAssetWhitelisted { asset: asset };
    let query_response: bool = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: bank_address.to_string(),
            msg: to_binary(&query_msg).unwrap(),
        }))
        .unwrap();

    query_response
}
