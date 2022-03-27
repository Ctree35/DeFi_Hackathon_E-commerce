use std::iter::Map;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Coin, Uint128, from_binary, AllBalanceResponse, Addr, CosmosMsg, BankMsg, Pair};
use cosmwasm_std::OverflowOperation::Add;
use cosmwasm_std::{coin, coins};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{AddressesResponse, BalanceResponse, ExecuteMsg, GoodsResponse, InstantiateMsg, OrderDetailResponse, OrdersResponse, QueryMsg, ShippingFeesResponse};

use crate::state::{State, STATE, Goods, GoodsStatus, GOODS_LIST, ORDER_LIST, SHIPPING_FEE_MATRIX, Order, OrderStatus, ShipperBid};
use crate::helper::{assert_sent_sufficient_coin, merge_coin};
// use serde::de::Unexpected::Map;
use crate::state::GoodsStatus::{Available, Ordered, Returned, Sold};
use cosmwasm_std::Order::Ascending;
use crate::ContractError::Unauthorized;
use crate::state::OrderStatus::{Bidding, Confirmed, Disputed, DisputingBroken, DisputingUnsatisfied, Setup, Shipping, WaitingAddressUpload};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:defi_ecommerce";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        order_cnt: 0,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    // SHIPPING_FEE
    let cities = vec!["Montreal", "Ottawa", "Toronto"];
    for c1 in cities.iter() {
        for c2 in cities.iter() {
            if c1 == c2 {
                SHIPPING_FEE_MATRIX.save(deps.storage, (c1, c2), &coin(5, "LUNA"))?;
            }
            else {
                SHIPPING_FEE_MATRIX.save(deps.storage, (c1, c2), &coin(10, "LUNA"))?;
            }
        }
    }
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Post {name, price, denom, area} => try_post(deps, info, &name, price, &denom, &area),
        ExecuteMsg::Buy {name, area} => try_buy(deps, info, &name, &area),
        ExecuteMsg::Reset {name, price} => try_reset(deps, info, &name, price),
        ExecuteMsg::TakeOrder { id, pub_key, price} => try_take_order(deps, info, id, pub_key, price),
        ExecuteMsg::ChooseBid {id, shipper} => try_choose_bid(deps, info, id, shipper),
        ExecuteMsg::UploadAddress { id, address_enc } => try_upload_address(deps, info, id, address_enc),
        ExecuteMsg::Confirm { id } => try_confirm(deps, info, id),
        ExecuteMsg::DisputeBroken { id } => try_dispute_broken(deps, info, id),
        ExecuteMsg::DisputeUnsatisfied { id } => try_dispute_unsatisfied(deps, info, id),
        ExecuteMsg::DisputeConfirm { id} => try_dispute_confirm(deps, info, id)
        // _ => unimplemented!()

    }
}

pub fn try_post(deps: DepsMut, info: MessageInfo, name: &str, price: u32, denom: &str, area: &str) -> Result<Response, ContractError> {
    let good = Goods {
        name: String::from(name),
        seller: info.sender,
        price: coin(Uint128::from(price).u128(), String::from(denom)),
        area: String::from(area),
        status: GoodsStatus::Available
    };
    GOODS_LIST.save(deps.storage, name, &good)?;
    Ok(Response::new().add_attribute("method", "try_post"))
}

pub fn try_buy(deps: DepsMut, info: MessageInfo, name: &str, area: &str) -> Result<Response, ContractError> {
    let mut good = GOODS_LIST.load(deps.storage, name)?;
    if good.status != Available {
        return Err(ContractError::GoodsNotAvailable {});
    }
    assert_sent_sufficient_coin(&info.funds, vec![good.clone().price])?;
    good.status = Ordered;
    let update_good = |d: Option<Goods>| -> StdResult<Goods> {
        match d {
            Some(_) => Ok(good.clone()),
            None => unimplemented!(),
        }
    };
    GOODS_LIST.update(deps.storage, name, update_good)?;
    let order = Order {
        id: STATE.load(deps.storage)?.order_cnt.clone(),
        buyer: info.sender.clone(),
        seller: good.clone().seller,
        goods: good.clone(),
        price: good.clone().price,
        buyer_address: String::from(area),
        shipper_bids: vec![],
        shipping_fee: SHIPPING_FEE_MATRIX.load(deps.storage, (&good.clone().area, area))?,
        shipper: Addr::unchecked("Dummy_Shipper"),
        shipper_key: Default::default(),
        buyer_addr_enc: Default::default(),
        seller_addr_enc: Default::default(),
        status: OrderStatus::Setup
    };
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.order_cnt += 1;
        Ok(state)
    })?;
    ORDER_LIST.save(deps.storage, &order.id.to_string(), &order)?;

    Ok(Response::new().add_attribute("method", "try_buy"))
}

pub fn try_reset(deps: DepsMut, info: MessageInfo, name: &str, price: u32) -> Result<Response, ContractError> {
    let mut good = GOODS_LIST.load(deps.storage, name)?;
    if good.status != Available {
        return Err(ContractError::GoodsNotAvailable {});
    }
    if good.seller != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    good.price.amount = Uint128::from(price);
    let update_good = |d: Option<Goods>| -> StdResult<Goods> {
        match d {
            Some(_) => Ok(good.clone()),
            None => unimplemented!(),
        }
    };
    GOODS_LIST.update(deps.storage, name, update_good)?;
    Ok(Response::new().add_attribute("method", "try_reset"))
}

pub fn try_take_order(deps: DepsMut, info: MessageInfo, id: u32, pub_key: String, price: Coin) -> Result<Response, ContractError> {
    let mut order = ORDER_LIST.load(deps.storage, &id.to_string())?;
    if order.status != Setup && order.status != Bidding {
        return Err(ContractError::OrderNotAvailable {});
    }
    assert_sent_sufficient_coin(&info.funds, vec![order.clone().price])?;
    order.status = Bidding;
    let bid = ShipperBid {
        shipper: info.sender,
        pub_key: pub_key,
        price: price
    };
    order.shipper_bids.push(bid);
    let update_order = |d: Option<Order>| -> StdResult<Order> {
        match d {
            Some(_) => Ok(order.clone()),
            None => unimplemented!(),
        }
    };
    ORDER_LIST.update(deps.storage, &id.to_string(), update_order)?;

    Ok(Response::new().add_attribute("method", "try_take_order"))
}

pub fn try_choose_bid(deps: DepsMut, info: MessageInfo, id: u32, shipper: String) -> Result<Response, ContractError> {
    let mut order = ORDER_LIST.load(deps.storage, &id.to_string())?;
    if order.status != Bidding {
        return Err(ContractError::OrderNotAvailable {});
    }
    if info.sender != order.buyer {
        return Err(ContractError::Unauthorized {});
    }
    match order.shipper_bids.iter().find(|x| x.shipper == Addr::unchecked(shipper.clone())) {
        Some(x) => {
            order.status = WaitingAddressUpload;
            order.shipper = x.shipper.clone();
            order.shipper_key = x.pub_key.clone();
            order.shipping_fee = x.price.clone();
            let update_order = |d: Option<Order>| -> StdResult<Order> {
                match d {
                    Some(_) => Ok(order.clone()),
                    None => unimplemented!(),
                }
            };
            ORDER_LIST.update(deps.storage, &id.to_string(), update_order)?;

            Ok(Response::new().add_attribute("method", "try_choose_bid"))
        }
        None => {
            Err(ContractError::ShipperNotFound {})
        }
    }
}

pub fn try_upload_address(deps: DepsMut, info: MessageInfo, id: u32, address_enc: String) -> Result<Response, ContractError> {
    let mut order = ORDER_LIST.load(deps.storage, &id.to_string())?;
    if order.status != WaitingAddressUpload {
        return Err(ContractError::OrderNotAvailable {});
    }
    if order.buyer == info.sender {
        order.buyer_addr_enc = address_enc;
    }
    else if order.seller == info.sender {
        order.seller_addr_enc = address_enc;
    }
    else {
        return Err(ContractError::Unauthorized {});
    }
    if order.buyer_addr_enc != String::default() && order.seller_addr_enc != String::default() {
        order.status = Shipping;
    }
    let update_order = |d: Option<Order>| -> StdResult<Order> {
        match d {
            Some(_) => Ok(order.clone()),
            None => unimplemented!(),
        }
    };
    ORDER_LIST.update(deps.storage, &id.to_string(), update_order)?;
    Ok(Response::new().add_attribute("method", "try_upload_address"))
}

pub fn try_confirm(deps: DepsMut, info: MessageInfo, id: u32) -> Result<Response, ContractError> {
    let mut order = ORDER_LIST.load(deps.storage, &id.to_string())?;
    if order.status != Shipping {
        return Err(ContractError::OrderNotAvailable {});
    }
    if order.buyer != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    order.status = Confirmed;
    let mut good = GOODS_LIST.load(deps.storage, &order.goods.name)?;
    good.status = Sold;
    order.goods = good.clone();
    let update_good = |d: Option<Goods>| -> StdResult<Goods> {
        match d {
            Some(_) => Ok(good.clone()),
            None => unimplemented!(),
        }
    };
    let update_order = |d: Option<Order>| -> StdResult<Order> {
        match d {
            Some(_) => Ok(order.clone()),
            None => unimplemented!(),
        }
    };
    ORDER_LIST.update(deps.storage, &id.to_string(), update_order)?;
    GOODS_LIST.update(deps.storage, &order.goods.name, update_good)?;
    Ok(Response::new()
        .add_attribute("method", "try_confirm")
        .add_message(CosmosMsg::Bank(BankMsg::Send { to_address: order.seller.into_string(), amount: vec![order.price] }))
        .add_message(CosmosMsg::Bank(BankMsg::Send { to_address: order.shipper.into_string(), amount: vec![order.shipping_fee] })))
}

pub fn try_dispute_broken(deps: DepsMut, info: MessageInfo, id: u32) -> Result<Response, ContractError> {
    let mut order = ORDER_LIST.load(deps.storage, &id.to_string())?;
    if order.status != Shipping {
        return Err(ContractError::OrderNotAvailable {});
    }
    if order.buyer != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    order.status = DisputingBroken;
    let update_order = |d: Option<Order>| -> StdResult<Order> {
        match d {
            Some(_) => Ok(order.clone()),
            None => unimplemented!(),
        }
    };
    ORDER_LIST.update(deps.storage, &id.to_string(), update_order)?;
    Ok(Response::new().add_attribute("method", "try_dispute_broken"))
}

pub fn try_dispute_unsatisfied(deps: DepsMut, info: MessageInfo, id: u32) -> Result<Response, ContractError> {
    let mut order = ORDER_LIST.load(deps.storage, &id.to_string())?;
    if order.status != Shipping {
        return Err(ContractError::OrderNotAvailable {});
    }
    if order.buyer != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    order.status = DisputingUnsatisfied;
    let update_order = |d: Option<Order>| -> StdResult<Order> {
        match d {
            Some(_) => Ok(order.clone()),
            None => unimplemented!(),
        }
    };
    ORDER_LIST.update(deps.storage, &id.to_string(), update_order)?;
    Ok(Response::new().add_attribute("method", "try_dispute_unsatisfied"))
}

pub fn try_dispute_confirm(deps: DepsMut, info: MessageInfo, id: u32) -> Result<Response, ContractError> {
    let mut order = ORDER_LIST.load(deps.storage, &id.to_string())?;
    if order.status != DisputingBroken && order.status != DisputingUnsatisfied {
        return Err(ContractError::OrderNotAvailable {});
    }
    if order.seller != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let mut res = Response::new().add_attribute("method", "try_dispute_confirm");
    res = match order.status {
        DisputingBroken => {
            res.add_message(CosmosMsg::Bank(BankMsg::Send { to_address: order.clone().buyer.into_string(), amount: merge_coin(vec![order.clone().price], coins(order.clone().shipping_fee.amount.checked_mul(Uint128::from(2u32)).unwrap().u128(), order.clone().shipping_fee.denom)) }))
                .add_message(CosmosMsg::Bank(BankMsg::Send { to_address: order.clone().seller.into_string(), amount: vec![order.clone().price] }))
        },
        DisputingUnsatisfied => {
            res.add_message(CosmosMsg::Bank(BankMsg::Send { to_address: order.clone().shipper.into_string(), amount: coins(order.clone().shipping_fee.amount.checked_mul(Uint128::from(2u32)).unwrap().u128(), order.clone().shipping_fee.denom) }))
                .add_message(CosmosMsg::Bank(BankMsg::Send { to_address: order.clone().buyer.into_string(), amount: vec![order.clone().price] }))
        },
        _ => unimplemented!()
    };
    order.status = Disputed;
    let mut good = GOODS_LIST.load(deps.storage, &order.goods.name)?;
    good.status = Returned;
    order.goods = good.clone();
    let update_good = |d: Option<Goods>| -> StdResult<Goods> {
        match d {
            Some(_) => Ok(good.clone()),
            None => unimplemented!(),
        }
    };
    let update_order = |d: Option<Order>| -> StdResult<Order> {
        match d {
            Some(_) => Ok(order.clone()),
            None => unimplemented!(),
        }
    };
    ORDER_LIST.update(deps.storage, &id.to_string(), update_order)?;
    GOODS_LIST.update(deps.storage, &order.goods.name, update_good)?;
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
//        QueryMsg::GetOrderDetail {id} => to_binary(&query_order_detail(deps, id)?),
//        QueryMsg::GetAddresses {id} => to_binary(&query_addresses(deps, id)?),
    match msg {
        QueryMsg::GetGoods {} => to_binary(&query_goods(deps)?),
        QueryMsg::GetOrders {} => to_binary(&query_orders(deps)?),
        QueryMsg::GetShippingFees {} => to_binary(&query_shipping_fees(deps)?),
        QueryMsg::GetOrderDetail {id} => to_binary(&query_order_detail(deps, id)?),
        QueryMsg::GetAddresses {id} => to_binary(&query_address(deps, id)?),
        QueryMsg::GetBalance {} => to_binary(&query_balance(deps, env)?),

    }
}

pub fn query_goods(deps: Deps) -> StdResult<GoodsResponse>{
    // let state = STATE.load(deps.storage)?;
    let good_list: StdResult<Vec<_>> = GOODS_LIST.range(deps.storage, None, None, Ascending).collect();
    let good_list = good_list.unwrap();
    let goods = good_list.iter().map(|x| x.1.clone()).collect();

    Ok(GoodsResponse{goods: {goods}})
}

pub fn query_orders(deps: Deps) -> StdResult<OrdersResponse> {
    let order_list: StdResult<Vec<_>> = ORDER_LIST.range(deps.storage, None, None, Ascending).collect();
    let order_list = order_list.unwrap();
    let orders = order_list.iter().map(|x| x.1.clone()).collect();

    Ok(OrdersResponse{orders: {orders}})
}

pub fn query_shipping_fees(deps: Deps) -> StdResult<ShippingFeesResponse> {
    let shipping_fee_matrix: StdResult<Vec<_>> = SHIPPING_FEE_MATRIX.range(deps.storage, None, None, Ascending).collect();
    let shipping_fee_matrix = shipping_fee_matrix.unwrap();
    let shipping_fees = shipping_fee_matrix.iter().map(|x| x.1.clone()).collect();

    Ok(ShippingFeesResponse{shipping_fees: {shipping_fees}})
}

pub fn query_order_detail(deps: Deps, id: u32) -> StdResult<OrderDetailResponse> {
    let order_list: StdResult<Vec<_>> = ORDER_LIST.range(deps.storage, None, None, Ascending).collect();
    let order_list = order_list.unwrap();
    let order = order_list.iter().find(|&x| String::from_utf8(x.clone().0).unwrap() == id.to_string());
    let order = match order {
        Some((_, o)) => o.clone(),
        None => unimplemented!()
    };

    Ok(OrderDetailResponse{order: {order}})
}

pub fn query_address(deps: Deps, id: u32) -> StdResult<AddressesResponse> {
    let order_list: StdResult<Vec<_>> = ORDER_LIST.range(deps.storage, None, None, Ascending).collect();
    let order_list = order_list.unwrap();
    let order = order_list.iter().find(|&x| String::from_utf8(x.clone().0).unwrap() == id.to_string());
    let buyer = match order {
        Some((_, o)) => o.clone().buyer,
        None => unimplemented!()
    };
    let seller = match order {
        Some((_, o)) => o.clone().seller,
        None => unimplemented!()
    };

    Ok(AddressesResponse{buyer: buyer.into_string(), seller: seller.into_string()})
}

pub fn query_balance(deps: Deps, env: Env) -> StdResult<BalanceResponse> {
    let balance = deps.querier.query_all_balances(env.contract.address).unwrap();

    Ok(BalanceResponse{balance})
}

#[cfg(test)]
mod tests {
    // use core::panicking::panic;
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn test_post() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Post {
            name: String::from("TV"),
            price: 200,
            denom: String::from("LUNA"),
            area: String::from("Montreal")
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//        // it worked, let's query the state
//        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//        let value: CountResponse = from_binary(&res).unwrap();
//        assert_eq!(17, value.count);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGoods {}).unwrap();
        let value: GoodsResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
    }

    #[test]
    fn test_buy() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Post {
            name: String::from("TV"),
            price: 200,
            denom: String::from("LUNA"),
            area: String::from("Montreal")
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg2 = ExecuteMsg::Buy {
            name: String::from("TV"),
            area: String::from("Montreal")
        };

        let info2 = mock_info("buyer", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info2, msg2).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOrders {}).unwrap();
        let value: OrdersResponse = from_binary(&res).unwrap();
        println!("{:?}", value);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAddresses {id: 0u32}).unwrap();
        let value: AddressesResponse = from_binary(&res).unwrap();
        println!("{:?}", value);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOrderDetail {id: 0u32}).unwrap();
        let value: OrderDetailResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
    }

    #[test]
    fn test_reset() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Post {
            name: String::from("TV"),
            price: 200,
            denom: String::from("LUNA"),
            area: String::from("Montreal")
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGoods {}).unwrap();
        let value: GoodsResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
        assert_eq!(Uint128::from(200u32), value.goods[0].price.amount);

        let msg2 = ExecuteMsg::Reset {
            name: String::from("TV"),
            price: 20
        };
        let info2 = mock_info("creator_fake", &coins(1000, "earth"));

        let res = execute(deps.as_mut(), mock_env(), info2, msg2.clone());
        match res {
            Err(ContractError::Unauthorized {}) => {},
            _ => panic!("Not buyer, not authorized!")
        }
        let _res = execute(deps.as_mut(), mock_env(), info, msg2);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGoods {}).unwrap();
        let value: GoodsResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
        assert_eq!(Uint128::from(20u32), value.goods[0].price.amount);
    }

    #[test]
    fn test_take_order() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Post {
            name: String::from("TV"),
            price: 200,
            denom: String::from("LUNA"),
            area: String::from("Montreal")
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg2 = ExecuteMsg::Buy {
            name: String::from("TV"),
            area: String::from("Montreal")
        };

        let info2 = mock_info("buyer", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info2, msg2).unwrap();

        let msg3 = ExecuteMsg::TakeOrder {
            id: 0,
            pub_key: String::from("rsa1"),
            price: coin(10, "LUNA")
        };
        let info3 = mock_info("shipper1", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info3, msg3).unwrap();

        let msg4 = ExecuteMsg::TakeOrder {
            id: 0,
            pub_key: String::from("rsa2"),
            price: coin(10, "LUNA")
        };
        let info4 = mock_info("shipper2", &coins(5000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info4, msg4).unwrap();

        let msg33 = ExecuteMsg::ChooseBid {
            id: 0,
            shipper: String::from("shipper1")
        };
        let info33 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info33, msg33).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOrders {}).unwrap();
        let value: OrdersResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
    }

    #[test]
    fn test_upload_address() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("seller", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Post {
            name: String::from("TV"),
            price: 200,
            denom: String::from("LUNA"),
            area: String::from("Montreal")
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg2 = ExecuteMsg::Buy {
            name: String::from("TV"),
            area: String::from("Montreal")
        };

        let info2 = mock_info("buyer", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info2, msg2).unwrap();

        let msg3 = ExecuteMsg::TakeOrder {
            id: 0,
            pub_key: String::from("rsa1"),
            price: coin(10, "LUNA")
        };
        let info3 = mock_info("shipper", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info3, msg3).unwrap();

        let msg33 = ExecuteMsg::ChooseBid {
            id: 0,
            shipper: String::from("shipper")
        };
        let info33 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info33, msg33).unwrap();

        let msg4 = ExecuteMsg::UploadAddress {
            id: 0,
            address_enc: String::from("my address")
        };
        let info4 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info4, msg4).unwrap();

        let msg5 = ExecuteMsg::UploadAddress {
            id: 0,
            address_enc: String::from("my address")
        };
        let info5 = mock_info("seller", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info5, msg5).unwrap();
    }

    #[test]
    fn test_confirm() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("seller", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Post {
            name: String::from("TV"),
            price: 200,
            denom: String::from("LUNA"),
            area: String::from("Montreal")
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg2 = ExecuteMsg::Buy {
            name: String::from("TV"),
            area: String::from("Montreal")
        };

        let info2 = mock_info("buyer", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info2, msg2).unwrap();

        let msg3 = ExecuteMsg::TakeOrder {
            id: 0,
            pub_key: String::from("rsa1"),
            price: coin(10, "LUNA")
        };
        let info3 = mock_info("shipper", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info3, msg3).unwrap();

        let msg33 = ExecuteMsg::ChooseBid {
            id: 0,
            shipper: String::from("shipper")
        };
        let info33 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info33, msg33).unwrap();

        let msg4 = ExecuteMsg::UploadAddress {
            id: 0,
            address_enc: String::from("my address")
        };
        let info4 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info4, msg4).unwrap();

        let msg5 = ExecuteMsg::UploadAddress {
            id: 0,
            address_enc: String::from("my address")
        };
        let info5 = mock_info("seller", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info5, msg5).unwrap();

        let msg6 = ExecuteMsg::Confirm {
            id: 0,
        };
        let info6 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info6, msg6).unwrap();
    }

    #[test]
    fn test_dispute() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("seller", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Post {
            name: String::from("TV"),
            price: 200,
            denom: String::from("LUNA"),
            area: String::from("Montreal")
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg2 = ExecuteMsg::Buy {
            name: String::from("TV"),
            area: String::from("Montreal")
        };

        let info2 = mock_info("buyer", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info2, msg2).unwrap();

        let msg31 = ExecuteMsg::TakeOrder {
            id: 0,
            pub_key: String::from("rsa1"),
            price: coin(10, "LUNA")
        };
        let info31 = mock_info("shipper1", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info31, msg31).unwrap();

        let msg32 = ExecuteMsg::TakeOrder {
            id: 0,
            pub_key: String::from("rsa1"),
            price: coin(8, "LUNA")
        };
        let info32 = mock_info("shipper2", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info32, msg32).unwrap();

        let msg33 = ExecuteMsg::ChooseBid {
            id: 0,
            shipper: String::from("shipper1")
        };
        let info33 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info33, msg33).unwrap();

        let msg4 = ExecuteMsg::UploadAddress {
            id: 0,
            address_enc: String::from("my address")
        };
        let info4 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info4, msg4).unwrap();

        let msg5 = ExecuteMsg::UploadAddress {
            id: 0,
            address_enc: String::from("my address")
        };
        let info5 = mock_info("seller", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info5, msg5).unwrap();

        let msg6 = ExecuteMsg::DisputeUnsatisfied {
            id: 0,
        };
        let info6 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info6, msg6).unwrap();

        let msg7 = ExecuteMsg::DisputeConfirm {
            id: 0,
        };
        let info7 = mock_info("seller", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info7, msg7).unwrap();

        let msg8 = QueryMsg::GetBalance {};
        let res = query(deps.as_ref(), mock_env(),  msg8).unwrap();
        let value: BalanceResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
    }
}