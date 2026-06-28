## lesson-gsd-frontmatter-body-separators | 2026-06-28 14:14

1. Date: 2026-06-28
2. What went wrong: A GSD summary used standalone `---` body separators after YAML frontmatter. The GSD frontmatter parser scans all `--- ... ---` blocks and selected the last body pair, so lifecycle validation ignored the real frontmatter and failed.
3. Preventive rule: In GSD artifacts and other frontmatter-parsed Markdown, use standalone `---` only for the opening and closing YAML frontmatter delimiters at the top of the file. Use headings or `***` for body breaks instead. Markdown table separator rows such as `| --- |` remain valid.
4. Trigger signal to catch it earlier: Lifecycle validation reports missing frontmatter fields even though the file visibly has them near the top, or a Markdown artifact has more than two standalone `---` lines.
