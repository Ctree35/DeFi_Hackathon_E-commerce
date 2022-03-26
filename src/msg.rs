use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Uint128;
use crate::state::{Goods, Order};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Post {name: String, price: u32, denom: String, location: String},
    Buy {name: String, location: String},
    Reset { name: String, price: u32 },
    TakeOrder {id: u32, pub_key: u32},
    UploadAddress {id: u32, address_enc: u32},
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
    GetShippingFees {},
    // GetOrderDetail {id: u32},
    // GetAddresses {id: u32}
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
pub struct ShippingFeesResponse {
    pub shipping_fees: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderDetailResponse {
    pub order: Order,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddressesResponse {
    pub buyer: u32,
    pub seller: u32
}
