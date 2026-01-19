---
title: "Simple YubiKey-backed SSH certificate authority"
date: 2026-01-18
tags: ["self-hosting", "security", "yubikey"]
---

I've been quite happy using SSH certificates in my homelab for a few months now. In this post, I'll describe the problems they solve, how to set up a CA using a YubiKey, and how to use it with clients and servers that only have OpenSSH.

# Why SSH certificates?

SSH certificates solve two common problems with SSH key management:

1. **Key registration:** With traditional SSH keys, you have to copy every public key to every server. If you have $N$ keys and $M$ servers, that's $N \times M$ copies to manage. 
2. **Trust on first use:** When you first connect to a server, you *(should!)* manually verify the host key fingerprint. 

```text
The authenticity of host '...' can't be established.
ED25519 key fingerprint is: SHA256:...
This key is not known by any other names.
Are you sure you want to continue connecting (yes/no/[fingerprint])?
```

With SSH certificates, you instead create a certificate authority (CA). Then once the servers and clients both trust the CA the process changes:

1. When the **server** trusts the CA, then any user key signed by the CA can be used to log in as the corresponding user. When you want to add a new key for a user, sign it once and give the user that certificate. Nothing needs to change on the server! 

2. When the **client** trusts the CA, then the host key is automatically verified against it. When you set up a new host and sign its host key with the CA, no changes or manual confirmation upon connection are needed on each client.

Usually both are done together, but if you just want to grant user access or verify hosts you can do that one side only.

# Operations

There are three operations we can perform:

1. Create a new certificate authority
2. Sign user keys
3. Sign host keys

These correspond to the [scripts in my GitHub repo](https://github.com/nicolaschan/ssh-ca/tree/master/scripts) and you can [read the README](https://github.com/nicolaschan/ssh-ca) for streamlined instructions complete with a Nix flake for the dependencies. This post will show how to manually use the underlying `ykman` and `ssh-keygen` commands to interact with the YubiKey and create/sign keys.

## Prerequisites

Only OpenSSH is needed on clients and servers to use the certificates.

For signing:
- A YubiKey that supports PIV and the `ykman` utility
- The `opensc-pkcs11.so` library for `ssh-keygen` to use the YubiKey-stored key (usually comes with the OpenSC package)

## Creating the YubiKey-backed CA

The CA is very powerful since it can sign any user or host key. It consists of a private key and public key pair. To keep the private key safe, we'll generate it in a YubiKey PIV slot. 

First, generate a new key on the YubiKey in slot `9a` (you could change the slot):

```bash
ykman piv keys generate --algorithm ECCP256 9a pubkey.pem
```

The key is stored on the YubiKey, so you can get the public key `pubkey.pem` back if you ever lose it:

```bash
ykman piv keys export 9a pubkey.pem
```

Now using the CA's `pubkey.pem`, generate a certificate for the CA and convert it to SSH format. I don't think the `$ca_domain` here matters much since it's just a label for the CA. I just set it to my domain.

```bash
ca_domain="example.com"
ykman piv certificates generate --subject "CN=$ca_domain SSH CA,O=$ca_domain" 9a pubkey.pem
ssh-keygen -i -m PKCS8 -f pubkey.pem > ssh-ca.pub
```

Now `ssh-ca.pub` is the public key of the CA in SSH format.

On servers, copy it to `/etc/ssh/ssh-ca.pub` and add the path to `/etc/ssh/sshd_config`. This will allow any user key signed by this CA to log in.

```text:/etc/ssh/sshd_config
TrustedUserCAKeys /etc/ssh/ssh-ca.pub
```

On clients, append it to `~/.ssh/known_hosts`. This will automatically verify host keys signed by this CA. You can scope it down by changing the `*` to a specific domain or IP range.

```bash
echo "@cert-authority * $(cat ssh-ca.pub)" >> ~/.ssh/known_hosts
```

## Signing host keys

To sign a host key, you will need the host's public key file to sign (e.g., `/etc/ssh/ssh_host_ed25519_key.pub`) and the hostnames that clients will use to connect to the server. 

```bash
first_host="server.example.com" # label for the certificate
all_hosts="server.example.com,alias.example.com" # valid hostnames for the certificate
keyfile="ssh_host_ed25519_key.pub"
ssh-keygen -D /path/to/opensc-pkcs11.so -s ssh-ca.pub -I "$first_host" -h -n "$all_hosts" -V +52w "$keyfile"
```

You can sign multiple hostnames in one certificate in `$all_hosts` if clients might use different hostnames for the same server. This is the list that clients will use to check if they are connecting to the real server. Optionally, update the `-V` flag to set your own expiration time for the certificate.

You'll get the file `ssh_host_ed25519_key-cert.pub` which you can place next to the host key on the server (e.g., `/etc/ssh/ssh_host_ed25519_key-cert.pub`). 

Then add the following line to `/etc/ssh/sshd_config`:

```text:/etc/ssh/sshd_config
HostCertificate /etc/ssh/ssh_host_ed25519_key-cert.pub
```

Make sure to restart the SSH server after adding the certificate.

## Granting user access

To allow a user to sign in with `~/.ssh/id_ed25519.pub`, sign the SSH public key with the CA: 

```bash
username="alice" # can be a comma-separated list of usernames
hostname="laptop.example.com" # just to label the certificate
pubkey="$HOME/.ssh/id_ed25519.pub"
ssh-keygen -D /path/to/opensc-pkcs11.so -s ssh-ca.pub -I "$username@$hostname" -n "$username" -V +52w "$pubkey"
```

By default, the certificate is valid for the username specified (and a comma-separated list of usernames is also supported). Optionally, set `-V` to change the expiration time.

This will create `~/.ssh/id_ed25519-cert.pub` which should be placed next to the private key. It might be necessary to update the ssh-agent using `ssh-add ~/.ssh/id_ed25519` to pick up the new certificate:

```bash
ssh-add ~/.ssh/id_ed25519
ssh-add -l
# Look for one ending in `-CERT`
# 256 SHA256:AbCdEfGhIjKlMnOpQrStUvWxYzAbCdEfGhIjKlMnOpQ alice@local (ED25519)
# 256 SHA256:AbCdEfGhIjKlMnOpQrStUvWxYzAbCdEfGhIjKlMnOpQ alice@local (ED25519-CERT)
```

# Taking this further

I like the simplicity of these scripts and that clients don't need any additional tooling beyond OpenSSH. I think it's a good improvement over traditional SSH key management but there are a few gaps: certificate rotation and management.

In the event of a key compromise you need to revoke the certificates for that key somehow. Rather than maintaining a revocation list or rotating the CA, the best approach is short-lived certificates that will expire automatically. But for this the keys will expire so often that you'll need some kind of just-in-time signing service. My understanding is that this is what [smallstep SSH](https://smallstep.com/product/ssh/) does but it requires additional tooling on the client side.

Certificates will also expire according to whatever date you set, so you will need some way to track upcoming expirations and rotate them ahead of time. At a small scale, this can be done manually but a larger system will need more tooling and automation around this process.

