"""Stable source identity for the host regression executable; no volatile clock input."""
def _reader_test_environment_impl(ctx):
    environment = ctx.actions.declare_file("reader-test-build.env")
    provenance = ctx.actions.declare_file("reader-test-build.json")
    ctx.actions.run(
        executable = ctx.attr.writer[DefaultInfo].files_to_run,
        arguments = [ctx.info_file.path, environment.path, provenance.path],
        inputs = [ctx.info_file],
        outputs = [environment, provenance],
        tools = [ctx.attr.writer[DefaultInfo].files_to_run],
        env = {"BAZEL_BINDIR": ctx.bin_dir.path},
        mnemonic = "BootstrapReaderTestIdentity",
    )
    return [DefaultInfo(files = depset([environment])), OutputGroupInfo(provenance = depset([provenance]))]

reader_test_environment = rule(
    implementation = _reader_test_environment_impl,
    attrs = {"writer": attr.label(default = Label("//scripts:usb_bootstrap_test_provenance_writer"), executable = True, cfg = "exec")},
)
