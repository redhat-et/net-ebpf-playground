#!/bin/sh
set -e

up() {
	ip link add podrouter type dummy
	ip link set up podrouter
	ip addr add 192.168.10.1 dev podrouter

	ip netns add pod1
	ip link add pod1 type veth peer name pod1-int
	ip link set pod1-int netns pod1 name eth0
	ip netns exec pod1 ip addr add 192.168.10.2/32 dev eth0
	ip netns exec pod1 ip link set up eth0
	ip netns exec pod1 ip route add 192.168.10.1 dev eth0
	ip netns exec pod1 ip route add default via 192.168.10.1
	ip link set up pod1
	ip route add 192.168.10.2/32 dev pod1

	ip netns add pod2
	ip link add pod2 type veth peer name pod2-int
	ip link set pod2-int netns pod2 name eth0
	ip netns exec pod2 ip addr add 192.168.10.2/32 dev eth0
	ip netns exec pod2 ip link set up eth0
	ip netns exec pod2 ip route add 192.168.10.1 dev eth0
	ip netns exec pod2 ip route add default via 192.168.10.1
	ip link set up pod2
	ip route add 192.168.10.3/32 dev pod2

	ip netns add pod3
	ip link add pod3 type veth peer name pod3-int
	ip link set pod3-int netns pod3 name eth0
	ip netns exec pod3 ip addr add 192.168.10.3/32 dev eth0
	ip netns exec pod3 ip link set up eth0
	ip netns exec pod3 ip route add 192.168.10.1 dev eth0
	ip netns exec pod3 ip route add default via 192.168.10.1
	ip link set up pod3
	ip route add 192.168.10.4/32 dev pod3
}

down() {
	ip netns del pod1
	ip netns del pod2
	ip netns del pod3
	ip link del podrouter
}

if test "$(id -u)" -ne "0"; then
	echo "please run as root"
	exit 1
fi

case $1 in
	"up")
		up
		;;
	"down")
		down
		;;
	*)
		echo "please provide a command. 'up' or 'down'"
		exit 1
esac