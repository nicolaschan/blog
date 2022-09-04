---
title: "Open Wireless"
date: 2022-09-04T13:06:21-07:00
tags: ["self-hosting", "networking"]
---

I don't have a password on my WiFi.

The internet is the 21<sup>st</sup> century library. Everyone should have access to a public library. Similarly, everyone should have access to public internet. I have more bandwidth available than I need. Sharing my extra bandwidth is the neighborly thing to do. For more details, [openwireless.org](https://openwireless.org) aligns with my views.

# Isn't that _dangerous_?

Most tech savvy people will say you need a password on your WiFi. There are some risks but we can manage them:

1. **Traffic on an open network is unencrypted.** 

    You should be visiting HTTPS sites anyway. If you _really_ have unencrypted traffic that you cannot avoid, then use a VPN on your device. You can also set up a separate secured wireless network for that.

2. **Unknown users may use too much bandwidth and cause service degredation.** 

    Apply a quality-of-service (QoS) rule on the open wireless network. Personally, I don't mitigate this since I have plenty of bandwidth. I have never had anyone abuse it to the point of degrading service.

3. **Unknown users may do bad things from your IP.** 

    Tunnel open wireless traffic over a VPN. That way it doesn't originate from your IP address.

4. **There are unsecured devices (like printers) on the network.** 

    Put them on a separate VLAN. Personally, I don't care. If a neighbor really needs to print something out, go for it.

In practice, I have found that these concerns are overblown. I have never had an issue with anyone abusing the open wireless service. Your router's built-in guest network is probably fine, barring extreme circumstances.

# Do people actually use it?

I've run an open network (SSID [openwireless.org](https://openwireless.org)) in Berkeley and currently run one in San Francisco. Both were broadcast from my apartment in a multi-dwelling unit. 

In Berkeley, we got _lots_ of users, even some consistent ones. I think this really helped out our cash-strapped student neighbors. We had gigabit fiber so we never noticed any service degredation even without any QoS rules. I also tunneled the guest traffic over [Mullvad VPN](https://mullvad.net/en/) (details on how to do this are below).

In San Francisco, I seldom get any users. &#x1F625; This is probably because my neighbors have their own networks. So they have no need for open wireless. I still run it just in case they need a backup if their internet goes down or for any passersby on the street.

# Tech details

I have a [Ubiquiti EdgeRouter-12](https://store.ui.com/collections/operator-isp-infrastructure/products/edgerouter-12) and [U6 Long Range](https://store.ui.com/products/u6-lr-us).

Sending guest traffic over the Mullvad VPN looks something like below. It consists of:
- WireGuard interface
- Routing table
- Guest firewall
- Guest VLAN

With these in place, anyone on the guest VLAN should have their traffic tunneled over the VPN.

## WireGuard interface
I use the [wireguard-vyatta-ubnt](https://github.com/WireGuard/wireguard-vyatta-ubnt) package and configure the interface according to Mullvad's config they provide.
```
interfaces {
    wireguard {
        wg1 {
            address 10.68.22.87/32
            address fc00:bbbb:bbbb:bb01::5:1656/128
            mtu 1420
            peer FSd0QIqNsLGf+B/IqQzg9wyjKpfVwXiy/P9vt8Zylmg= {
                allowed-ips 0.0.0.0/0
                allowed-ips ::0/0
                endpoint 198.54.134.146:51820
            }
            private-key /config/auth/mullvad/priv.key
            route-allowed-ips false
        }
    }
}
```

## Routing table
I found that the IPv6 route `interface-route6 ::/0` sometimes doesn't work after a router reboot. To fix this, I have give it a "kick" by switching the `distance` from `1` to `2` or vice-versa. If anyone knows how to properly fix this, please let me know!

```
protocols {
    static {
        table 96 {
            description "table for mullvad"
            interface-route 0.0.0.0/0 {
                next-hop-interface wg1 {
                    distance 2
                }
            }
            interface-route6 ::/0 {
                next-hop-interface wg1 {
                    distance 1
                }
            }
            route 0.0.0.0/0 {
                blackhole {
                    distance 255
                }
            }
            route6 ::/0 {
                blackhole {
                    distance 255
                }
            }
        }
    }
}
```

## Guest firewall

These are just some standard firewall rules to keep the guest network users isolated. The `modify MULLVAD_RULE` is for applying the new routes defined above.
```
firewall {
    group {
        network-group LAN_NETWORKS {
            description "Reserved IP addresses"
            network 10.0.0.0/8
            network 100.64.0.0/10
            network 169.254.0.0/16
            network 172.16.0.0/12
            network 192.0.0.0/24
            network 192.0.2.0/24
            network 192.88.99.0/24
            network 192.168.0.0/16
            network 198.18.0.0/15
            network 198.51.100.0/24
            network 203.0.113.0/24
            network 224.0.0.0/4
            network 233.252.0.0/24
            network 240.0.0.0/4
        }
    }

    ipv6-modify MULLVAD_RULEv6 {
        description "modify rule for mullvad"
        rule 20 {
            action modify
            modify {
                table 96
            }
            source {
                address fd26:6382:ed54:ea9c::/64
            }
        }
    }

    ipv6-name GUESTv6_IN {
        default-action accept
        description "guest to lan/wan"
        rule 10 {
            action drop
            description "drop guest to lan"
            destination {
                address fc00::/7
            }
            protocol all
        }
    }

    ipv6-name GUESTv6_LOCAL_RESTRICTIVE {
        default-action drop
        description "guest to router (restrictive)"
        rule 10 {
            action accept
            description "allow DNS"
            destination {
                port 53
            }
            log disable
            protocol tcp_udp
        }
        rule 20 {
            action accept
            description "allow DHCPv6"
            destination {
                port 547
            }
            log disable
            protocol udp
        }
        rule 30 {
            action accept
            description "allow ICMPv6"
            log disable
            protocol icmpv6
        }
    }

    modify MULLVAD_RULE {
        description "modify rule for mullvad"
        rule 20 {
            action modify
            modify {
                table 96
            }
            source {
                address 198.51.100.1/24
            }
        }
    }

    name GUEST_IN {
        default-action accept
        rule 20 {
            action drop
            description "drop guest to lan"
            destination {
                group {
                    network-group LAN_NETWORKS
                }
            }
            log disable
            protocol all
        }
        rule 30 {
            action drop
            description "drop guest to guest"
            destination {
                address 198.51.100.1/24
            }
            log disable
            protocol all
        }
    }

    name GUEST_LOCAL {
        default-action accept
        description "guest to router"
        rule 10 {
            action accept
            description "allow wireguard"
            destination {
                port 56594
            }
            log disable
            protocol udp
        }
        rule 20 {
            action accept
            description "allow DNS"
            destination {
                port 53
            }
            log disable
            protocol tcp_udp
        }
        rule 30 {
            action accept
            description "allow DHCP"
            destination {
                port 67
            }
            log disable
            protocol udp
        }
        rule 40 {
            action accept
            description "allow ping"
            log disable
            protocol icmp
        }
    }
}
```

## Guest VLAN

Finally, define the guest VLAN and apply the rules.

```
switch switch0 {
     vif 96 {
         address 192.168.96.1/24
         address fd26:6382:ed54:ea9c::1/64
         description Guest
         firewall {
             in {
                 ipv6-modify MULLVAD_RULEv6
                 ipv6-name GUESTv6_IN
                 modify MULLVAD_RULE
                 name GUEST_IN
             }
             local {
                 ipv6-name GUESTv6_LOCAL_RESTRICTIVE
                 name GUEST_LOCAL
             }
         }
         ipv6 {
             dup-addr-detect-transmits 1
             router-advert {
                 cur-hop-limit 64
                 link-mtu 0
                 managed-flag true
                 max-interval 600
                 other-config-flag false
                 prefix fd26:6382:ed54:ea9c::/64 {
                     autonomous-flag true
                     on-link-flag true
                     valid-lifetime 2592000
                 }
                 reachable-time 0
                 retrans-timer 0
                 send-advert true
             }
         }
     }
}
```