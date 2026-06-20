# Use Reference Breadcrumbs at Module and Behavior Boundaries

Rust-owned modules and behavior-specific functions should include concise reference breadcrumbs to the upstream source paths, functions, and parity checklist anchors that define the behavior being matched. Breadcrumbs should appear at module and behavior boundaries, not on every translated line, so comments remain useful audit links without turning the Rust implementation into a line-by-line C mirror.
