use log::{debug, warn};
use net_route::Route as NetRoute;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Contains information that describes a route in the local computer's Ipv4 or Ipv6 routing table.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Route {
    /// Network address of the destination. `0.0.0.0` with a prefix of `0` is considered a default route.
    pub destination: IpAddr,

    /// Length of network prefix in the destination address.
    pub prefix: u8,

    /// The address of the next hop of this route.
    ///
    /// On macOS, this must be `Some` if ifindex is `None`
    pub gateway: Option<IpAddr>,

    /// The index of the local interface through which the next hop of this route may be reached.
    ///
    /// On macOS, this must be `Some` if gateway is `None`
    pub ifindex: Option<u32>,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            destination: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            prefix: 0,
            gateway: None,
            ifindex: None,
        }
    }
}

impl Route {
    /// Create a route that matches a given destination network.
    pub fn new(
        destination: IpAddr,
        prefix: u8,
        gateway: Option<IpAddr>,
        ifindex: Option<u32>,
    ) -> Self {
        Self {
            destination,
            prefix,
            gateway,
            ifindex,
        }
    }

    /// Get the netmask covering the network portion of the destination address.
    pub fn mask(&self) -> IpAddr {
        match self.destination {
            IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::from(
                u32::MAX.checked_shl(32 - self.prefix as u32).unwrap_or(0),
            )),
            IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::from(
                u128::MAX.checked_shl(128 - self.prefix as u32).unwrap_or(0),
            )),
        }
    }
}

impl From<NetRoute> for Route {
    fn from(route: NetRoute) -> Self {
        Self {
            destination: route.destination,
            prefix: route.prefix,
            gateway: route.gateway,
            ifindex: route.ifindex,
        }
    }
}

#[maybe_async::maybe_async]
pub async fn get_default_route() -> Option<Route> {
    if let Ok(handle) = net_route::Handle::new() {
        let default_route = handle.default_route().await;

        if let Ok(route) = default_route {
            if let Some(route) = route {
                debug!("Default route:\n{:?}", route);
                return Some(route.into());
            }
        }
    } else {
        warn!("Failed to get route handle");
    }
    warn!("No default route");
    None
}

#[maybe_async::maybe_async]
pub async fn get_all_routes() -> Option<Vec<Route>> {
    if let Ok(handle) = net_route::Handle::new() {
        let routes = handle.list().await;

        if let Ok(routes) = routes {
            debug!("All routes:\n{:?}", routes);
            return Some(routes.into_iter().map(|r| r.into()).collect());
        }
    } else {
        warn!("Failed to get route handle");
    }
    warn!("Failed to get all routes");
    None
}
