// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::config::NodeConfig;
use aptos_types::{
    network_address::{NetworkAddress, Protocol},
    transaction::Transaction,
};
use get_if_addrs::get_if_addrs;
use rand::rngs::OsRng;
use rand::Rng;
use std::net::{TcpListener, TcpStream};
use std::ops::Range;

const MAX_PORT_RETRIES: u16 = 1000;
// Using non-ephemeral ports, to avoid conflicts with OS-selected ports (i.e., bind on port 0)
const RANDOM_PORT_RANGE: Range<u16> = 10000..30000;

/// Return a non-ephemeral, available port. On unix systems, the port returned will be in the
/// TIME_WAIT state ensuring that the OS won't hand out this port for some grace period.
/// Callers should be able to bind to this port given they use SO_REUSEADDR.
pub fn get_available_port() -> u16 {
    for _ in 0..MAX_PORT_RETRIES {
        if let Ok(port) = get_random_port() {
            return port;
        }
    }

    panic!("Error: could not find an available port");
}

fn get_random_port() -> ::std::io::Result<u16> {
    // Choose a random port and try to bind
    let port = OsRng.gen_range(RANDOM_PORT_RANGE.start, RANDOM_PORT_RANGE.end);
    let listener = TcpListener::bind(("localhost", port))?;
    let addr = listener.local_addr()?;

    // Create and accept a connection (which we'll promptly drop) in order to force the port
    // into the TIME_WAIT state, ensuring that the port will be reserved from some limited
    // amount of time (roughly 60s on some Linux systems)
    let _sender = TcpStream::connect(addr)?;
    let _incoming = listener.accept()?;

    Ok(addr.port())
}

/// Extracts one local non-loopback IP address, if one exists. Otherwise returns None.
pub fn get_local_ip() -> Option<NetworkAddress> {
    get_if_addrs().ok().and_then(|if_addrs| {
        if_addrs
            .iter()
            .find(|if_addr| !if_addr.is_loopback())
            .map(|if_addr| NetworkAddress::from(Protocol::from(if_addr.ip())))
    })
}

pub fn get_available_port_in_multiaddr(is_ipv4: bool) -> NetworkAddress {
    let ip_proto = if is_ipv4 {
        Protocol::Ip4("0.0.0.0".parse().unwrap())
    } else {
        Protocol::Ip6("::1".parse().unwrap())
    };
    NetworkAddress::from_protocols(vec![ip_proto, Protocol::Tcp(get_available_port())]).unwrap()
}

pub fn get_genesis_txn(config: &NodeConfig) -> Option<&Transaction> {
    config.execution.genesis.as_ref()
}
