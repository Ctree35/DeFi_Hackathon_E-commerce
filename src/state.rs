use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub balance: Vec<Coin>,
    pub goods_list: Vec<Box<Goods>>,
    pub order_list: Vec<Box<Order>>,
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