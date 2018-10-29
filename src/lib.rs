extern crate nix;

use std::process::{Command, Stdio};
//use nix::sys::socket::socket;
//use nix::sys::socket::AddressFamily;
//use nix::sys::socket::SockType;
//use nix::sys::socket::SockFlag;
use nix::sys::socket::SockAddr;
use nix::sys::socket::InetAddr;

#[derive(Clone)]
pub enum Status {
    Working(String),
    Error(String),
    Good(String),
}

fn get_address(iface: &str) -> Option<nix::sys::socket::InetAddr> {
    let addrs = nix::ifaddrs::getifaddrs().unwrap();
    for ifaddr in addrs {
        if ifaddr.interface_name == iface {
            match ifaddr.address {
                Some(address) => {
                    match address {
                        SockAddr::Inet(InetAddr::V4(addr)) => return Some(InetAddr::V4(addr)),
                        _ => {}
                    }
                },
                None => { }
            }
        }
    }
    None
}

fn can_ping(address: &str) -> bool {
    let status = Command::new("/bin/ping")
        .arg("-c").arg("2").arg("-i").arg(".5").arg(address)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match status {
        Ok(status) => status.success(),
        Err(_) => false
    }
}

fn test_dhcp(iface: &str) -> bool {
//# dhcpcd -T wlan0 -t 2
    let status = Command::new("/sbin/dhcpcd")
        .arg("-T").arg(iface).arg("-t").arg("8")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match status {
        Ok(status) => status.success(),
        Err(_) => false
    }
}

fn has_address(iface: &str) -> bool {
    match get_address(&iface) {
        Some(_address) => true,
        None => false
    }
}

pub fn test_network(ifname: &str, tx: std::sync::mpsc::Sender<Status>) -> Result<(), ()> {
    if has_address(&ifname) {
        //println!("Interface {} appears to be active.", ifname);
        tx.send(Status::Working(format!("Interface {} appears to be active.", ifname))).unwrap();
    } else {
        tx.send(Status::Error(format!("FAILED, local interface does not have an IP!"))).unwrap();
        return Err(());
    }

    if test_dhcp(&ifname) {
        tx.send(Status::Working(format!("DHCP works!"))).unwrap();
    } else {
        tx.send(Status::Working(format!("DHCP Failed, local LAN/WiFi will not work correctly!"))).unwrap();
        if can_ping("192.168.1.1") {
            tx.send(Status::Error(format!("Firewall is UP."))).unwrap();
        } else {
            tx.send(Status::Error(format!("Firewall is DOWN- Fix the firewall!"))).unwrap();
        }
        return Err(());
    }
    if can_ping("google.com") {
        tx.send(Status::Good(format!("Internet is working."))).unwrap();
        return Ok(());
    } else {
        tx.send(Status::Working(format!("Failed to ping google.com, network issues!"))).unwrap();
    }
    if can_ping("8.8.8.8") {
        tx.send(Status::Error(format!("Pinged 8.8.8.8, this indicates DNS has failed!"))).unwrap();
        return Err(());
    } else {
        tx.send(Status::Working(format!("Failed to ping 8.8.8.8, this indicates internet is down- will test LAN now!"))).unwrap();
        if can_ping("192.168.1.1") {
            tx.send(Status::Error(format!("Firewall is UP, this indicates a cable modem/comcast issue!"))).unwrap();
            return Err(());
        } else {
            tx.send(Status::Error(format!("Firewall is DOWN, fix the firewall!"))).unwrap();
            return Err(());
        }
    }
}

