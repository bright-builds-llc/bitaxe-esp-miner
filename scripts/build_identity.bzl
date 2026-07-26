"""Cache-correct transport for the canonical Rust build-provenance materializer."""

def _build_provenance_impl(ctx):
    stamp = ctx.actions.declare_file(ctx.label.name + ".stamp")
    sdkconfig_defaults = ctx.actions.declare_file(ctx.label.name + ".sdkconfig.defaults")
    build_timestamp_utc = ctx.actions.declare_file(ctx.label.name + ".build-timestamp-utc")
    args = ctx.actions.args()
    args.add("materialize-build-provenance")
    args.add("--status-file", ctx.info_file)
    args.add("--volatile-status-file", ctx.version_file)
    args.add("--stamp-out", stamp)
    args.add("--sdkconfig-defaults-out", sdkconfig_defaults)
    args.add("--build-timestamp-out", build_timestamp_utc)

    ctx.actions.run(
        executable = ctx.executable._materializer,
        arguments = [args],
        inputs = [ctx.info_file, ctx.version_file] + ctx.files.srcs,
        outputs = [stamp, sdkconfig_defaults, build_timestamp_utc],
        tools = [ctx.executable._materializer],
        mnemonic = "BitaxeBuildProvenance",
        progress_message = "Materializing canonical Bitaxe build provenance",
    )

    return [
        DefaultInfo(files = depset([stamp, sdkconfig_defaults])),
        OutputGroupInfo(
            build_timestamp_utc = depset([build_timestamp_utc]),
            sdkconfig_defaults = depset([sdkconfig_defaults]),
            stamp = depset([stamp]),
        ),
    ]

build_provenance = rule(
    implementation = _build_provenance_impl,
    attrs = {
        "srcs": attr.label_list(allow_files = True),
        "_materializer": attr.label(
            default = Label("//tools/xtask:xtask"),
            executable = True,
            cfg = "exec",
        ),
    },
)
