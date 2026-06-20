# Use Bazel as Automation Graph and Just as Command Surface

Bazel is the canonical automation graph for build, test, package, flash, and release-shaped workflows, while `just` provides the human-friendly command surface. Early Bazel targets may invoke repo-owned scripts or ESP-IDF/Cargo-compatible steps where direct Bazel rules are immature, but important workflows should still be represented as Bazel targets so local use and CI follow the same dependency graph.
