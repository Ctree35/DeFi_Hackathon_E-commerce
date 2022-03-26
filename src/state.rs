use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub balance: Vec<Coin>,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Goods {
    pub name: String,
    pub seller: Addr,
    pub price: Coin,
    pub location: String,
    pub status: GoodsStatus
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Order {
    pub id: u32,
    pub buyer: Addr,
    pub seller: Addr,
    pub goods: Goods,
    pub price: Coin,
    pub shipping_fee: Coin,
    pub shipper: Addr,
    pub shipper_key: u32,
    pub status: OrderStatus
}

//pub struct Location {
////    pub latitude: i32,
////    pub longitude: i32
////}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GoodsStatus {
    Available,
    Shipping,
    Sold
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum OrderStatus {
    Setup,
    WaitingAddressUpload,
    Shipping,
    Confirmed,
    Disputing,
    Disputed
}

pub const STATE: Item<State> = Item::new("state");
pub const GOODS_LIST: Map<&str, Goods> = Map::new("goods_list");
pub const ORDER_LIST: Map<&str, Order> = Map::new("order_list");