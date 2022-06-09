Experiments with eBPF for K8s
=============================

This is a work in progress

# Prerequisites

A Fedora 36 Workstation with:

1. `dnf install elfutils-libelf-devel`
2. LLVM and Clang
3. `skopeo`, `umoci` and `runc`

# Topology

Set up a test topology - where eth0 is your internet connected interface

```sh
sudo ./topology.sh up eth0
```

Tear down the topology
```sh
sudo ./topology.sh down
```

Clear the image cache
```sh
sudo ./topoligy.sh clean
```

# Build Programs

```sh
make
```

# Attach a program

XDP

```sh
sudo ip link set pod1 xdp object ./.output/pass.bpf.o section xdp/pass
```

TC

```sh
sudo tc qdisc add dev pod1 clsact
sudo tc filter add dev pod1 ingress prio 1 handle 1 bpf object-file .output/filter.bpf.o direct-action 
```
