use std::net::{Ipv4Addr, Ipv6Addr};

use axum::extract::Query;

#[derive(serde::Deserialize)]
pub struct DestParams {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

pub async fn dest(Query(params): Query<DestParams>) -> String {
    let from_octets = params.from.octets();
    let key_octets = params.key.octets();
    let to: Ipv4Addr = [
        from_octets[0].wrapping_add(key_octets[0]),
        from_octets[1].wrapping_add(key_octets[1]),
        from_octets[2].wrapping_add(key_octets[2]),
        from_octets[3].wrapping_add(key_octets[3]),
    ]
    .into();

    to.to_string()
}

#[derive(serde::Deserialize)]
pub struct KeyParams {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

pub async fn key(Query(params): Query<KeyParams>) -> String {
    let from_octets = params.from.octets();
    let to_octets = params.to.octets();
    let key: Ipv4Addr = [
        to_octets[0].wrapping_sub(from_octets[0]),
        to_octets[1].wrapping_sub(from_octets[1]),
        to_octets[2].wrapping_sub(from_octets[2]),
        to_octets[3].wrapping_sub(from_octets[3]),
    ]
    .into();

    key.to_string()
}

#[derive(serde::Deserialize)]
pub struct DestV6Params {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

pub async fn dest_v6(Query(params): Query<DestV6Params>) -> String {
    ipv6_xor(params.from, params.key).to_string()
}

#[derive(serde::Deserialize)]
pub struct KeyV6Params {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

pub async fn key_v6(Query(params): Query<KeyV6Params>) -> String {
    ipv6_xor(params.from, params.to).to_string()
}

fn ipv6_xor(first: Ipv6Addr, second: Ipv6Addr) -> Ipv6Addr {
    let first_octets = first.octets();
    let second_octets = second.octets();
    let result_octets: [u8; 16] = first_octets
        .iter()
        .zip(second_octets.iter())
        .map(|(first, second)| first ^ second)
        .collect::<Vec<u8>>()
        .try_into()
        .expect("vector must have exactly 16 elements");

    result_octets.into()
}
