use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub balance: Vec<Coin>,
    pub order_cnt: u32,
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
    pub buyer_address: String,
    pub shipping_fee: Coin,
    pub shipper: Addr,
    pub shipper_key: u32,
    pub buyer_addr_enc: u32,
    pub seller_addr_enc: u32,
    pub status: OrderStatus
}

//pub struct Location {
////    pub latitude: i32,
////    pub longitude: i32
////}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GoodsStatus {
    Available,
    Ordered,
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
pub const SHIPPING_FEE_MATRIX: Map<(&str, &str), u32> = Map::new("shipping_fee_matrix");
