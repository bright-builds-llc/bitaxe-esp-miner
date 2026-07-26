#!/usr/bin/env bash

durability_log_has_terminal_ready() {
	[[ $# -eq 1 ]] || return 2
	local log_path="$1"
	[[ -f "$log_path" ]] || return 1
	local final_line
	final_line="$(tail -n 1 -- "$log_path")"
	[[ "$final_line" == "usb_session: ready" ]]
}
