---
title: "Home Lighting on Self-Hosted Kubernetes"
date: 2022-09-04T09:43:14-07:00
draft: true
tags: ["kubernetes", "self-hosting", "zigbee"]
---

In my SF apartment I control my lights using [Zigbee2MQTT](https://www.zigbee2mqtt.io/), [Mosquitto (MQTT)](https://mosquitto.org/man/mqtt-7.html), and [openHAB](https://www.openhab.org/) all running on Kubernetes. Here's the how and why for this nonsense.

# But why?

You could probably get similar features with various smart home hubs. Philosophically, I am opposed to my lights relying on services outside of my control. My goals are:

- Everything can run locally and if the internet is down.
- No hard dependencies on third party services.
- Control all properties of the lights programmatically. (On/off, color, etc.)
- Automate deployment as much as possible.
- Learn Kubernetes.

When it's all working I can control my lights from anywhere using a VPN. I can also use other Zigbee devices like smart plugs to turn things on/off when I'm away. OpenHAB plugins open up [a](https://www.openhab.org/addons/bindings/gpstracker/) [world](https://www.openhab.org/addons/bindings/minecraft/) [of](https://www.openhab.org/addons/bindings/astro/) [possibilities](https://www.openhab.org/addons/bindings/telegram/) for controlling the lights.

# Hardware

- [ConBee II Zigbee USB Gateway](https://phoscon.de/en/conbee2)
- [Philips Hue Zigbee Light Bulbs](https://www.philips-hue.com/en-hk/p/hue-white-and-color-ambiance-1-pack-e27/8718699719210#specifications)
- An old desktop PC (i5-6600K, 8GB DDR3)[^1]

# Tutorial

I run a single-node [k3s](https://k3s.io/) cluster managed by [Flux](https://fluxcd.io/) on [Arch Linux](https://archlinux.org/). All the [Kubernetes configs are defined on GitHub for reference](https://github.com/nicolaschan/infra/tree/master/apps/home/smart-home).

## Prerequisites

You should know the basics of Kubernetes. You should have a working Kubernetes cluster with a proxy like [Traefik](https://traefik.io/).

## Set up services

```bash
helm repo add k8s-at-home https://k8s-at-home.com/charts/
helm repo add kilip https://charts.itstoni.com/

helm repo update

helm install zigbee2mqtt k8s-at-home/zigbee2mqtt -f zibgee2mqtt-values.yaml
helm install mosquitto k8s-at-home/mosquitto -f mosquitto-values.yaml
helm install openhab kilip/openhab -f openhab-values.yaml
```

## Configure and use openHAB
TODO

[^1]: I tried using Raspberry Pi 4s, but they were _not_ powerful enough and would frequently drop messages causing shenanigans with the lights. Not ideal.