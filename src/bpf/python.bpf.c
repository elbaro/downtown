// bpftool btf dump file /sys/kernel/btf/vmlinux format c > vmlinux.h
// https://www.kernel.org/doc/html/latest/bpf/maps.html

#include "vmlinux.h"
// #include <bpf/bpf.h>
#include <bpf/bpf_helpers.h>
#include <bpf/usdt.bpf.h>
// #include "python.h"
#include "hist.h"
#include "str.h"

char LICENSE[] SEC("license") = "GPL";
volatile u64 FILTER_PATH_HASH = 0;

struct bpf_map_create_opts;
int bpf_map_create(enum bpf_map_type map_type,
		   const char *map_name,
		   __u32 key_size,
		   __u32 value_size,
		   __u32 max_entries,
		   const struct bpf_map_create_opts *opts);

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 1024);
	__type(key, u64);  // tid
	__type(value, u64); // indent
} indent_map SEC(".maps");

struct StackKey {
	u64 tid;
	u64 indent;
};

struct StackInfo {
	u64 start_timestamp;
	u64 lineno;
};

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 1024);
	__type(key, struct StackKey);
	__type(value, struct StackInfo);
} function_start SEC(".maps");

struct FilterKey {
	u64 lineno;
};

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 128);
	__type(key, struct FilterKey);
	__type(value, u64); // count
} filter_map SEC(".maps");

struct HistogramKey {
	struct FilterKey filter;
	u64 bucket;
};

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 128*512);
	__type(key, struct HistogramKey);
	__type(value, u64); // count
} latency_map SEC(".maps");

SEC("usdt")
int function__entry(struct pt_regs *ctx)
{
	u64 tid = bpf_get_current_pid_tgid();
	void *read = bpf_map_lookup_elem(&indent_map, &tid);
	u64 indent = (read) ? (*((u64*)(read)) + 1) : 1;
	bpf_map_update_elem(&indent_map, &tid, &indent, BPF_ANY);

	struct StackKey key = {
		.tid = tid,
		.indent = indent,
	};
	struct StackInfo info = {
		.start_timestamp = bpf_ktime_get_ns(),
		.lineno = 0,
	};
	bpf_usdt_arg(ctx, 2, (long*)(&info.lineno));
	bpf_map_update_elem(&function_start, &key, &info, BPF_ANY);

	return 0; 
}

SEC("usdt")
// int function_return(void* ctx, char* filename, char* funcname, int lineno)
int function__return(struct pt_regs *ctx)
{
	u64 tid = bpf_get_current_pid_tgid();
	void *read = bpf_map_lookup_elem(&indent_map, &tid);
	if (!read) { return 0; }
	u64 indent = *((u64*)read);
	//u64 indent = (read) ? (*((u64*)read)) : 10000;
	struct StackKey stack_key = {
		.tid = tid,
		.indent = indent,
	};
	indent--;
	bpf_map_update_elem(&indent_map, &tid, &indent, BPF_ANY);

	// filter with filename
	const char* arg_path;
	bpf_usdt_arg(ctx, 0, (long*)(&arg_path));
	u64 path_hash = 0;
	char path[128];

	for (int bound_loop=0;bound_loop<16;bound_loop++) {
		path[127] = 0;
		int copied = bpf_probe_read_user_str(path, sizeof(path), arg_path);
		for (int i = 0; i < copied; i++) {
			if (path[i] == '\0') { break; }
			path_hash = path_hash * 37 + path[i];
		}
		if (copied < 128 || path[127] == '\0') { break; }
	}
	

	if (path_hash != FILTER_PATH_HASH) {
		return 0;
	}

	// read start_timestamp and lineno
	read = bpf_map_lookup_elem(&function_start, &stack_key);
	if (!read) { return 0; }
	struct StackInfo* info = (struct StackInfo*)read;

	// filter with lineno
	if (bpf_map_lookup_elem(&filter_map, &info->lineno)) {
		const char* funcname;
		bpf_usdt_arg(ctx, 1, (long*)(&funcname));
		struct HistogramKey hist_key = {
			.filter.lineno = info->lineno,
			.bucket = hist4_bucket(bpf_ktime_get_ns() - info->start_timestamp),
		};

		read = bpf_map_lookup_elem(&latency_map, &hist_key);
		u64 count = (read) ? (*((u64*)read) + 1) : 1;
		bpf_map_update_elem(&latency_map, &hist_key, &count, BPF_ANY);
	}
	
	return 0;
}

SEC("usdt")
int gc_start()
{
	return 0;
}

SEC("usdt")
int gc_done()
{
	return 0; 
}
