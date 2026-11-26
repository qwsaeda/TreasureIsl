//! 本机可访问地址枚举（IPv4 / IPv6）——启动横幅打「直接可用的 URL」，不用 127.0.0.1。
//!
//! 用途：平台默认监听 `0.0.0.0`（IPv4 通配）时，用户仍需知道**本机实际网卡地址**
//! 才能从别的机器/手机访问。IPv6 另需注意 link-local 地址必须带 zone id
//! （浏览器 URL 形式 `http://[fe80::1%25eth0]:8787/`，`%` 需转义为 `%25`）。
//!
//! 实现要点（零新增依赖）：
//! - IPv4 首选地址 = UDP connect trick（连出去不收发包，`getsockname` 得本机源 IP）——跨平台、无副作用；
//! - IPv6 全局/链路本地地址 = 读 `/proc/net/if_inet6`（Linux）+ `/proc/net/dev` 取网卡名（zone id）；
//! - 非 Linux 平台退化为仅 IPv4（IPv6 用回环不可用则跳过）。

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket};

/// 枚举本机可访问地址（去重、IPv4 在前）。
///
/// 返回 `(ipv4, ipv6)`：IPv4 = 首选出口地址 + 其它网卡地址；IPv6 = 全局/链路本地（带 zone）。
#[must_use]
pub fn local_addresses() -> (Vec<String>, Vec<String>) {
    let mut v4: Vec<String> = Vec::new();
    let mut v6: Vec<String> = Vec::new();

    // ① IPv4 首选（UDP connect trick）：只设置目标地址，不发送任何数据。
    if let Ok(sock) = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)) {
        if sock.connect("192.0.2.1:9").is_ok() {
            if let Ok(addr) = sock.local_addr() {
                let ip = addr.ip();
                if !ip.is_loopback() && !ip.is_unspecified() {
                    v4.push(ip.to_string());
                }
            }
        }
    }
    // ② IPv6 首选（同类 trick，目标取文档前缀 2001:db8::，不真发包）。
    if let Ok(sock) = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)) {
        if sock.connect("[2001:db8::1]:9").is_ok() {
            if let Ok(addr) = sock.local_addr() {
                let ip = addr.ip();
                if let IpAddr::V6(v6ip) = ip {
                    if !v6ip.is_loopback() && !v6ip.is_unspecified() {
                        v6.push(v6ip.to_string());
                    }
                }
            }
        }
    }
    // ③ Linux：/proc/net/if_inet6 枚举（带 zone id）——只留主网卡，排除回环与容器网桥。
    let primary_v6 = primary_interface();
    for (ip, zone) in linux_ipv6_addrs() {
        let Some(z) = zone.clone() else { continue };
        if ip == "::1" {
            continue;
        }
        let is_primary = primary_v6.as_deref() == Some(z.as_str());
        if is_container_iface(&z) && !is_primary {
            continue;
        }
        let entry = format!("{ip}%{z}");
        if is_primary {
            v6.insert(0, entry);
        } else if !v6.contains(&entry) {
            v6.push(entry);
        }
    }
    // ④ Linux：网卡 IPv4 —— 只列**默认路由所在主网卡**（用户真正用来访问的地址）+ 其余非容器网卡。
    let primary = primary_interface();
    for (iface, ip) in linux_iface_ipv4() {
        if ip.starts_with("127.") {
            continue;
        }
        let is_primary = primary.as_deref() == Some(iface.as_str());
        if is_container_iface(&iface) && !is_primary {
            continue;
        }
        if is_primary {
            if !v4.contains(&ip) {
                v4.insert(0, ip);
            }
        } else if !v4.contains(&ip) {
            v4.push(ip);
        }
    }
    (v4, v6)
}

/// Linux：读 `/proc/net/if_inet6` → (地址字符串, 可选 zone=网卡名)。
///
/// 格式：`<32 hex 地址> <ifindex hex> <prefixlen hex> <scope hex> <flags> <网卡名>`
/// ⇒ **最后一列即网卡名**（zone id 所需），无须 ifindex 映射。
fn linux_ipv6_addrs() -> Vec<(String, Option<String>)> {
    let Ok(text) = std::fs::read_to_string("/proc/net/if_inet6") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(hex) = parts.next() else { continue };
        let Some(name) = parts.last() else { continue };
        if hex.len() != 32 {
            continue;
        }
        let mut bytes = [0u8; 16];
        let mut ok = true;
        for (i, chunk) in hex.as_bytes().chunks(4).enumerate() {
            let Ok(v) = u16::from_str_radix(std::str::from_utf8(chunk).unwrap_or(""), 16) else {
                ok = false;
                break;
            };
            let be = v.to_be_bytes();
            bytes[i * 2] = be[0];
            bytes[i * 2 + 1] = be[1];
        }
        if !ok {
            continue;
        }
        out.push((Ipv6Addr::from(bytes).to_string(), Some(name.to_owned())));
    }
    out
}

/// Linux：网卡名 → IPv4 地址（`ip -4 -o addr show`，每网卡取首个地址）。
fn linux_iface_ipv4() -> Vec<(String, String)> {
    let Ok(out) = std::process::Command::new("ip").args(["-4", "-o", "addr", "show"]).output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut out = Vec::new();
    for line in text.lines() {
        // 格式：`<idx>: <iface><空白>inet <addr>/<prefix> …`
        let Some((_, rest)) = line.split_once(": ") else {
            continue;
        };
        let mut tokens = rest.split_whitespace();
        let Some(iface) = tokens.next() else {
            continue;
        };
        let Some((_, addr)) = rest.split_once("inet ") else {
            continue;
        };
        let ip = addr.split('/').next().unwrap_or("").trim().to_owned();
        if !ip.is_empty() && !out.iter().any(|(_, existing)| *existing == ip) {
            out.push((iface.to_owned(), ip));
        }
    }
    out
}

/// Linux：默认路由（destination 全 0）所在网卡名 = 主网卡（用户实际访问用的那张）。
fn primary_interface() -> Option<String> {
    let text = std::fs::read_to_string("/proc/net/route").ok()?;
    for line in text.lines().skip(1) {
        let mut parts = line.split_whitespace();
        let iface = parts.next()?;
        let dest = parts.next()?;
        if dest == "00000000" {
            return Some(iface.to_owned());
        }
    }
    None
}

/// 容器网桥/虚拟网卡（docker0 / br-* / veth*）——不是用户从外部访问的地址。
fn is_container_iface(iface: &str) -> bool {
    iface.starts_with("docker") || iface.starts_with("br-") || iface.starts_with("veth")
}

/// 构造可点击 URL（IPv6 需方括号 + zone 的 `%` 转义为 `%25`）。
#[must_use]
pub fn url_for(ip: &str, port: u16) -> String {
    if ip.contains(':') {
        let zone = ip.split_once('%').map(|(_, z)| format!("%25{z}")).unwrap_or_default();
        let bare = ip.split('%').next().unwrap_or(ip);
        format!("http://[{bare}{zone}]:{port}/")
    } else {
        format!("http://{ip}:{port}/")
    }
}
