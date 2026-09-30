"""Embed the declared compiler input closure, never the later checkout."""
def _stamp_impl(ctx):
    output = ctx.actions.declare_file(ctx.label.name + ".env")
    args = ctx.actions.args()
    args.add(output.path)
    for file in ctx.files.inputs:
        args.add(file.short_path)
        args.add(file.path)
    ctx.actions.run(
        executable = ctx.executable.writer,
        arguments = [args],
        inputs = ctx.files.inputs,
        outputs = [output],
        tools = [ctx.attr.writer[DefaultInfo].files_to_run],
        env = {"BAZEL_BINDIR": "."},
        mnemonic = "VirtualCompilerInputs",
    )
    return [DefaultInfo(files = depset([output]))]
compiler_inputs = rule(
    implementation = _stamp_impl,
    attrs = {
        "inputs": attr.label_list(allow_files = True),
        "writer": attr.label(executable = True, cfg = "exec", mandatory = True),
    },
)
