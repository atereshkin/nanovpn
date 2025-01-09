# nanovpn
Smallest possible VPN in Rust

This is the smallest possible VPN that simply encapsulates IP packets as UDP
without any encryption or authentication. It's intended for learning (you can see
all parts involved in a VPN within two pages of code) and for prototyping other software.
If you need an unencrypted tunnel, just use GRE which is built into Linux kernel. 

## Running

```
$ sudo nanovpn <source port> <destination host> <destination port>
```

Nanovpn will use the first available tunN device. Now we need to set up addresses and 
routing. 
On machine A:
```
$ sudo ip addr add 10.11.0.1 dev tun0
$ sudo ip route add 10.11.0.0/16 dev tun0  
```
On machine B:
```
$ sudo ip addr add 10.11.0.2 dev tun0
$ sudo ip route add 10.11.0.0/16 dev tun0  
```

Now the machines can talk through the tunnel, e.g. on A:

```
$ ping 10.11.0.2 
```