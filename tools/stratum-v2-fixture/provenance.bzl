"""Bind the canonical fixture binary to Bazel's current source status."""

def _fixture_provenance_impl(ctx):
    receipt = ctx.actions.declare_file(ctx.attr.output_name)
    ctx.actions.run(
        executable = ctx.attr.writer[DefaultInfo].files_to_run,
        arguments = [ctx.info_file.path, ctx.executable.fixture.path, receipt.path],
        inputs = [ctx.info_file, ctx.executable.fixture],
        outputs = [receipt],
        tools = [ctx.attr.writer[DefaultInfo].files_to_run],
        env = {"BAZEL_BINDIR": ctx.bin_dir.path},
        mnemonic = "NoiseFixtureIdentity",
    )
    return [DefaultInfo(files = depset([receipt]))]

fixture_provenance = rule(
    implementation = _fixture_provenance_impl,
    attrs = {
        "fixture": attr.label(executable = True, cfg = "target", mandatory = True),
        "writer": attr.label(default = Label("//scripts:noise_fixture_identity_writer"), executable = True, cfg = "exec"),
        "output_name": attr.string(default = "noise-serial-build-identity.json"),
    },
)
