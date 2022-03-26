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
    Post {name: str, price: u32, denom: str, location: str},
    Buy {name: str, location: str},
    Reset { price: u32 },
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
    GetDistance {},
    GetOrderDetail {id: u32},
    GetAddresses {id: u32}
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
pub struct DistanceResponse {
    pub distance: u32,
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