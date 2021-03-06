use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Coin, Uint128};
use crate::state::{Goods, Order};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Post {name: String, price: u32, denom: String, seller_area: String},
    Buy {name: String, buyer_area: String},
    Reset { name: String, price: u32 },
    TakeOrder {id: u32, pub_key: String, price: Coin},
    ChooseBid {id: u32, shipper: String},
    UploadAddress {id: u32, address_enc: Vec<u8>},
    Confirm {id: u32},
    DisputeBroken {id: u32},
    DisputeUnsatisfied {id: u32},
    DisputeConfirm {id: u32}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetGoods {},
    GetOrders {},
    GetOrderDetail {id: u32},
    GetAddresses {id: u32},
    GetBalance {}
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GoodsResponse {
    pub goods: Vec<Goods>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrdersResponse {
    pub orders: Vec<Order>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderDetailResponse {
    pub order: Order,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddressesResponse {
    pub buyer: Vec<u8>,
    pub seller: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BalanceResponse {
    pub balance: Vec<Coin>
}
