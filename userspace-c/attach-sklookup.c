// SPDX-License-Identifier: (GPL-2.0-only OR BSD-2-Clause)
/* Copyright (c) 2020 Cloudflare */

#include <errno.h>
#include <error.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <linux/bpf.h>

#include "syscall.h"

int bpf(enum bpf_cmd cmd, union bpf_attr *attr, unsigned int size)
{
    return syscall(__NR_bpf, cmd, attr, size);
}

int main(int argc, char **argv)
{
	const char *prog_path;
	const char *link_path;
	const char *netns;
	union bpf_attr attr;
	int prog_fd, netns_fd, link_fd, err;
	char netns_path[50]; 

	if (argc != 4) {
		fprintf(stderr, "Usage: %s <prog path> <link path> <netns name>\n", argv[0]);
		exit(EXIT_SUCCESS);
	}

	prog_path = argv[1];
	link_path = argv[2];
	netns = argv[3];

	/* 1. Open the pinned BPF program */
	memset(&attr, 0, sizeof(attr));
	attr.pathname = (uint64_t) prog_path;
	attr.file_flags = BPF_F_RDONLY;

	prog_fd = bpf(BPF_OBJ_GET, &attr, sizeof(attr));
	if (prog_fd == -1)
		error(EXIT_FAILURE, errno, "bpf(OBJ_GET)");

	/* 2. Get an FD for this process network namespace (netns) */
	if(strcmp(netns,"self") != 0) {
		sprintf(netns_path, "/var/run/netns/%s", netns);
	} else {
		sprintf(netns_path, "/proc/self/ns/net");
	}

	printf("Attaching to network namespace: %s\n", netns_path);
	
	netns_fd = open(netns_path, O_RDONLY | O_CLOEXEC);
	if (netns_fd == -1)
		error(EXIT_FAILURE, errno, "open");

	/* 3. Attach BPF sk_lookup program to the (netns) with a BPF link */
	memset(&attr, 0, sizeof(attr));
	attr.link_create.prog_fd = prog_fd;
	attr.link_create.target_fd = netns_fd;
	attr.link_create.attach_type = BPF_SK_LOOKUP;
	attr.link_create.flags = 0;

	link_fd = bpf(BPF_LINK_CREATE, &attr, sizeof(attr));
	if (link_fd == -1)
		error(EXIT_FAILURE, errno, "bpf(LINK_CREATE)");

	/* 4. Pin the BPF link (otherwise would be destroyed on FD close) */
	memset(&attr, 0, sizeof(attr));
	attr.pathname = (uint64_t) link_path;
	attr.bpf_fd = link_fd;
	attr.file_flags = 0;

	err = bpf(BPF_OBJ_PIN, &attr, sizeof(attr));
	if (err)
		error(EXIT_FAILURE, errno, "bpf(OBJ_PIN)");

	close(link_fd);
	close(netns_fd);
	close(prog_fd);

	exit(EXIT_SUCCESS);
}
