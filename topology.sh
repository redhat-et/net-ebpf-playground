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
	create_pod 1

	ip netns add pod2
	ip link add pod2 type veth peer name pod2-int
	ip link set pod2-int netns pod2 name eth0
	ip netns exec pod2 ip addr add 192.168.10.2/32 dev eth0
	ip netns exec pod2 ip link set up eth0
	ip netns exec pod2 ip route add 192.168.10.1 dev eth0
	ip netns exec pod2 ip route add default via 192.168.10.1
	ip link set up pod2
	ip route add 192.168.10.3/32 dev pod2
	create_pod 2

	ip netns add pod3
	ip link add pod3 type veth peer name pod3-int
	ip link set pod3-int netns pod3 name eth0
	ip netns exec pod3 ip addr add 192.168.10.3/32 dev eth0
	ip netns exec pod3 ip link set up eth0
	ip netns exec pod3 ip route add 192.168.10.1 dev eth0
	ip netns exec pod3 ip route add default via 192.168.10.1
	ip link set up pod3
	ip route add 192.168.10.4/32 dev pod3
	create_pod 3
}

down() {
	set +e
	sudo runc delete pod1
	sudo runc delete pod2
	sudo runc delete pod3
	sudo rm -rf ./containers
	ip netns del pod1
	ip netns del pod2
	ip netns del pod3
	ip link del podrouter
}

create_pod() {
	pod="pod$1"
	if [ ! -f .output/rootfs.tar ]; then
		mkdir -p .output
		sudo docker run --name net-ebpf-playground fedora:35 /bin/bash
		sudo docker export net-ebpf-playground > .output/rootfs.tar
	fi
	bundle_dir="./containers/${pod}"
	mkdir -p "${bundle_dir}/rootfs"
	tar -xf .output/rootfs.tar -C "${bundle_dir}/rootfs"
	if [ ! -f .output/config.json ]; then
		runc spec -b .output
	fi
	jq --arg pod "$pod" '.linux.namespaces[1].path = "/var/run/netns/\($pod)" | .process.args[0] = "sleep" | .process.args[1] = "infinity" | .process.terminal = false' .output/config.json > "${bundle_dir}/config.json"
	sudo runc create -b "${bundle_dir}" "${pod}"
	sudo runc start "${pod}"
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
