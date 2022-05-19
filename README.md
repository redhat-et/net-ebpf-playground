Experiments with eBPF for K8s
=============================

This is a work in progress

# Prerequisites

1. `dnf install elfutils-libelf-devel`
2. LLVM
3. `docker` and `runc`

# Topology

Set up a test topology

```sh
sudo ./topology.sh up
```

Tear down the topology
```sh
sudo ./topology.sh down
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
