use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub order_cnt: u32,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Goods {
    pub name: String,
    pub seller: Addr,
    pub price: Coin,
    pub seller_area: String,
    pub status: GoodsStatus
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Order {
    pub id: u32,
    pub buyer: Addr,  // hash address on chain
    pub seller: Addr,  // hash address on chain
    pub goods: Goods,
    pub price: Coin,
    pub buyer_area: String,
    pub shipper_bids: Vec<ShipperBid>,
    pub shipping_fee: Coin,
    pub shipper: Addr,
    pub shipper_key: String,
    pub buyer_addr_enc: String,
    pub seller_addr_enc: String,
    pub status: OrderStatus
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ShipperBid {
    pub shipper: Addr,
    pub pub_key: String,
    pub price: Coin
}

//pub struct Location {
////    pub latitude: i32,
////    pub longitude: i32
////}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GoodsStatus {
    Available,
    Ordered,
    Sold,
    Returned
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum OrderStatus {
    Setup,
    Bidding,
    WaitingAddressUpload,
    Shipping,
    Confirmed,
    DisputingBroken,
    DisputingUnsatisfied,
    Disputed
}

pub const STATE: Item<State> = Item::new("state");
pub const GOODS_LIST: Map<&str, Goods> = Map::new("goods_list");
pub const ORDER_LIST: Map<&str, Order> = Map::new("order_list");
pub const SHIPPING_FEE_MATRIX: Map<(&str, &str), Coin> = Map::new("shipping_fee_matrix");
