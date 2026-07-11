#!/usr/bin/env bash
# Shared macOS/Linux process-group lifecycle helpers for bounded shell watchers.

PHASE_PROCESS_GROUP_PID=""
PHASE_PROCESS_GROUP_STATE_FILE="${PHASE_PROCESS_GROUP_STATE_FILE:-}"

phase_process_group_is_alive() {
	local pid="$1"
	[[ "$pid" =~ ^[0-9]+$ ]] || return 1
	kill -0 -- "-$pid" >/dev/null 2>&1
}

phase_process_is_alive() {
	local pid="$1"
	[[ "$pid" =~ ^[0-9]+$ ]] || return 1
	kill -0 "$pid" >/dev/null 2>&1
}

phase_process_group_start() {
	local ready_file="$1"
	shift
	(($# > 0)) || return 2

	rm -f "$ready_file"
	perl -MPOSIX=setsid -e '
		my $ready_file = shift @ARGV;
		my $state_file = shift @ARGV;
		if (length $state_file) {
			open my $state, ">", $state_file or die "open state file failed: $!\n";
			print {$state} "$$\n" or die "write state file failed: $!\n";
			close $state or die "close state file failed: $!\n";
		}
		defined(setsid()) or die "setsid failed: $!\n";
		open my $ready, ">", $ready_file or die "open readiness file failed: $!\n";
		print {$ready} "$$\n" or die "write readiness file failed: $!\n";
		close $ready or die "close readiness file failed: $!\n";
		exec {$ARGV[0]} @ARGV;
		die "exec failed: $!\n";
	' "$ready_file" "$PHASE_PROCESS_GROUP_STATE_FILE" "$@" &
	PHASE_PROCESS_GROUP_PID=$!

	local ready_pid
	for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
		if [[ -s "$ready_file" ]]; then
			ready_pid="$(sed -n '1p' "$ready_file")"
			rm -f "$ready_file"
			[[ "$ready_pid" == "$PHASE_PROCESS_GROUP_PID" ]] || return 1
			return 0
		fi
		phase_process_is_alive "$PHASE_PROCESS_GROUP_PID" || break
		sleep 0.01
	done

	phase_process_group_terminate "$PHASE_PROCESS_GROUP_PID" "process-group startup" >/dev/null 2>&1
	rm -f "$ready_file"
	return 1
}

phase_process_group_terminate() {
	local pid="$1"
	local label="$2"
	local signal_status

	[[ "$pid" =~ ^[0-9]+$ ]] || {
		printf '%s: invalid process-group pid\n' "$label" >&2
		return 1
	}

	if kill -TERM -- "-$pid" >/dev/null 2>&1; then
		signal_status=0
	else
		signal_status=$?
	fi
	if kill -TERM "$pid" >/dev/null 2>&1; then
		:
	fi
	if ((signal_status != 0)) && phase_process_group_is_alive "$pid"; then
		printf '%s: failed to signal process group %s\n' "$label" "$pid" >&2
	fi

	for _ in 1 2 3 4 5 6 7 8 9 10; do
		if ! phase_process_group_is_alive "$pid" && ! phase_process_is_alive "$pid"; then
			break
		fi
		sleep 0.1
	done

	if phase_process_group_is_alive "$pid" || phase_process_is_alive "$pid"; then
		if kill -KILL -- "-$pid" >/dev/null 2>&1; then
			:
		fi
		if kill -KILL "$pid" >/dev/null 2>&1; then
			:
		fi
	fi

	if wait "$pid" >/dev/null 2>&1; then
		:
	fi

	for _ in 1 2 3 4 5 6 7 8 9 10; do
		if ! phase_process_group_is_alive "$pid" && ! phase_process_is_alive "$pid"; then
			return 0
		fi
		sleep 0.1
	done

	printf '%s: process group %s is still alive after cleanup\n' "$label" "$pid" >&2
	return 1
}
