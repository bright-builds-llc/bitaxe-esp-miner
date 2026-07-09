#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const JOB_FRAME_LEN = 88;
const JOB_HEADER = 0x21;
const JOB_LENGTH_FIELD = 0x56;
const JOB_PAYLOAD_OFFSET = 4;

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.1-wire-field-compare.mjs --upstream <log> --rust <log> --out <report.md> [--upstream-source-work <report.json> --rust-source-work <report.json>]

Compares BM1366 job-frame fields from ignored upstream and Rust UART logs.
The report is redaction-safe: it records field verdicts, not raw bytes.
`);
  process.exit(exitCode);
}

function parseArgs(argv) {
  const args = new Map();
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") {
      usage(0);
    }

    if (!arg.startsWith("--")) {
      usage(1);
    }

    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      usage(1);
    }

    args.set(arg.slice(2), value);
    index += 1;
  }

  return {
    upstream: args.get("upstream"),
    rust: args.get("rust"),
    out: args.get("out"),
    upstreamSourceWork: args.get("upstream-source-work"),
    rustSourceWork: args.get("rust-source-work"),
  };
}

function readMaybeFile(filePath) {
  if (!filePath || !fs.existsSync(filePath)) {
    return null;
  }

  return fs.readFileSync(filePath, "utf8");
}

function parseHexLine(line) {
  const maybeRust = line.match(/asic_uart_trace=tx\s+len=(\d+)\s+hex=([0-9a-fA-F ]+)/);
  if (maybeRust) {
    return toBytes(maybeRust[2]);
  }

  const maybeUpstreamDebug = line.match(/(?:^|\s)(?:tx|rx):\s*\[([0-9a-fA-F ]+)\]/i);
  if (maybeUpstreamDebug) {
    return toBytes(maybeUpstreamDebug[1]);
  }

  if (!/SERIAL(?:TX|RX)_DEBUG|serial(?:tx|rx)|uart|BM1366/i.test(line)) {
    return [];
  }

  return toBytes(line);
}

function toBytes(text) {
  return [...text.matchAll(/\b[0-9a-fA-F]{2}\b/g)].map((match) =>
    Number.parseInt(match[0], 16),
  );
}

function findJobFrames(logText) {
  if (!logText) {
    return [];
  }

  const frames = [];
  for (const line of logText.split(/\r?\n/)) {
    const bytes = parseHexLine(line);
    if (bytes.length !== JOB_FRAME_LEN) {
      continue;
    }

    if (
      bytes[0] !== 0x55 ||
      bytes[1] !== 0xaa ||
      bytes[2] !== JOB_HEADER ||
      bytes[3] !== JOB_LENGTH_FIELD
    ) {
      continue;
    }

    frames.push(decodeJobFrame(bytes));
  }

  return frames;
}

function readMaybeSourceWorkReport(filePath) {
  if (!filePath || !fs.existsSync(filePath)) {
    return null;
  }

  try {
    const parsed = JSON.parse(fs.readFileSync(filePath, "utf8"));
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return { usable: false, reason: "source_work_report_invalid" };
    }
    const fingerprint = parsed.source_work_fingerprint;
    if (typeof fingerprint !== "string" || !/^[0-9a-f]{64}$/u.test(fingerprint)) {
      return { usable: false, reason: "source_work_fingerprint_missing" };
    }
    return {
      usable: true,
      fingerprint,
      label: path.basename(filePath),
    };
  } catch {
    return { usable: false, reason: "source_work_report_unreadable" };
  }
}

function sourceWorkAlignmentStatus(upstreamPath, rustPath) {
  const upstreamReport = readMaybeSourceWorkReport(upstreamPath);
  const rustReport = readMaybeSourceWorkReport(rustPath);

  if (!upstreamPath && !rustPath) {
    return {
      status: "not_proven",
      reportsProvided: false,
      blockedReasons: [],
      upstreamReport,
      rustReport,
    };
  }

  const blockedReasons = [];
  if (!upstreamReport) {
    blockedReasons.push("upstream_source_work_report_missing");
  } else if (!upstreamReport.usable) {
    blockedReasons.push(`upstream_${upstreamReport.reason}`);
  }

  if (!rustReport) {
    blockedReasons.push("rust_source_work_report_missing");
  } else if (!rustReport.usable) {
    blockedReasons.push(`rust_${rustReport.reason}`);
  }

  if (blockedReasons.length > 0) {
    return {
      status: "blocked_safe_prerequisite",
      reportsProvided: true,
      blockedReasons,
      upstreamReport,
      rustReport,
    };
  }

  if (upstreamReport.fingerprint !== rustReport.fingerprint) {
    return {
      status: "mismatch",
      reportsProvided: true,
      blockedReasons: ["source_work_fingerprint_mismatch"],
      upstreamReport,
      rustReport,
    };
  }

  return {
    status: "matched",
    reportsProvided: true,
    blockedReasons: [],
    upstreamReport,
    rustReport,
  };
}

function decodeJobFrame(bytes) {
  const payload = bytes.slice(JOB_PAYLOAD_OFFSET, JOB_PAYLOAD_OFFSET + 82);
  return {
    frame_length: bytes.length,
    header: bytes[2],
    length_field: bytes[3],
    job_id: payload[0],
    num_midstates: payload[1],
    starting_nonce: payload.slice(2, 6),
    nbits: payload.slice(6, 10),
    ntime: payload.slice(10, 14),
    merkle_root: payload.slice(14, 46),
    prev_block_hash: payload.slice(46, 78),
    version: payload.slice(78, 82),
  };
}

function equalField(left, right) {
  if (Array.isArray(left) && Array.isArray(right)) {
    return left.length === right.length && left.every((value, index) => value === right[index]);
  }

  return left === right;
}

function compareBestJobPair(upstreamFrames, rustFrames) {
  const fields = [
    "frame_length",
    "header",
    "length_field",
    "job_id",
    "num_midstates",
    "starting_nonce",
    "nbits",
    "ntime",
    "merkle_root",
    "prev_block_hash",
    "version",
  ];

  let bestComparison = null;
  for (let upstreamIndex = 0; upstreamIndex < upstreamFrames.length; upstreamIndex += 1) {
    for (let rustIndex = 0; rustIndex < rustFrames.length; rustIndex += 1) {
      const upstream = upstreamFrames[upstreamIndex];
      const rust = rustFrames[rustIndex];
      const rows = fields.map((field) => ({
        field,
        verdict: equalField(upstream[field], rust[field]) ? "match" : "mismatch",
      }));
      const mismatches = rows
        .filter((row) => row.verdict === "mismatch")
        .map((row) => row.field);
      const matchedFieldCount = fields.length - mismatches.length;

      if (!bestComparison || matchedFieldCount > bestComparison.matchedFieldCount) {
        bestComparison = {
          upstreamIndex,
          rustIndex,
          matchedFieldCount,
          comparedFieldCount: fields.length,
          rows,
          mismatches,
        };
      }
    }
  }

  return {
    status: bestComparison.mismatches.length === 0 ? "match" : "mismatch",
    ...bestComparison,
  };
}

function reportFor({
  upstreamPath,
  rustPath,
  upstreamText,
  rustText,
  upstreamSourceWorkPath,
  rustSourceWorkPath,
}) {
  const upstreamFrames = findJobFrames(upstreamText);
  const rustFrames = findJobFrames(rustText);
  const sourceWork = sourceWorkAlignmentStatus(upstreamSourceWorkPath, rustSourceWorkPath);
  const blockedReasons = [];

  if (!upstreamText) {
    blockedReasons.push("upstream_log_missing");
  } else if (upstreamFrames.length === 0) {
    blockedReasons.push("upstream_job_frame_missing");
  }

  if (!rustText) {
    blockedReasons.push("rust_log_missing");
  } else if (rustFrames.length === 0) {
    blockedReasons.push("rust_job_frame_missing");
  }

  blockedReasons.push(...sourceWork.blockedReasons);

  if (blockedReasons.length > 0) {
    return renderReport({
      status: "blocked_safe_prerequisite",
      upstreamPath,
      rustPath,
      upstreamSourceWorkPath,
      rustSourceWorkPath,
      upstreamFrames,
      rustFrames,
      blockedReasons,
      rows: [],
      mismatches: [],
      sourceWork,
    });
  }

  const comparison = compareBestJobPair(upstreamFrames, rustFrames);
  const dynamicWorkFields = new Set(["ntime", "merkle_root"]);
  const dynamicOnlyMismatch =
    comparison.mismatches.length > 0 &&
    comparison.mismatches.every((field) => dynamicWorkFields.has(field));
  const status =
    dynamicOnlyMismatch && sourceWork.status !== "matched"
      ? "mismatch_unconfirmed_dynamic_work_fields"
      : comparison.status;
  return renderReport({
    status,
    upstreamPath,
    rustPath,
    upstreamSourceWorkPath,
    rustSourceWorkPath,
    upstreamFrames,
    rustFrames,
    blockedReasons,
    rows: comparison.rows,
    mismatches: comparison.mismatches,
    sourceWork,
    selectedPair: {
      upstreamIndex: comparison.upstreamIndex,
      rustIndex: comparison.rustIndex,
      matchedFieldCount: comparison.matchedFieldCount,
      comparedFieldCount: comparison.comparedFieldCount,
    },
  });
}

function renderReport({
  status,
  upstreamPath,
  rustPath,
  upstreamSourceWorkPath,
  rustSourceWorkPath,
  upstreamFrames,
  rustFrames,
  blockedReasons,
  rows,
  mismatches,
  sourceWork,
  selectedPair = null,
}) {
  const sourceLabel = (value) => {
    if (!value) return "missing";
    return path.basename(value);
  };

  const lines = [
    "# Phase 28.1.1.1 BM1366 Wire Field Comparator Report",
    "",
    `comparison_status: ${status}`,
    `upstream_capture_status: ${upstreamFrames.length > 0 ? "job_frame_found" : "missing_or_unusable"}`,
    `rust_capture_status: ${rustFrames.length > 0 ? "job_frame_found" : "missing_or_unusable"}`,
    `source_work_alignment_status: ${sourceWork.status}`,
    "raw_bytes_committed: false",
    "credential_contents_read: false",
    `upstream_source_label: ${sourceLabel(upstreamPath)}`,
    `rust_source_label: ${sourceLabel(rustPath)}`,
    `upstream_source_work_label: ${sourceLabel(upstreamSourceWorkPath)}`,
    `rust_source_work_label: ${sourceLabel(rustSourceWorkPath)}`,
    `source_work_fingerprint_match: ${sourceWork.status === "matched"}`,
    `upstream_job_frame_count: ${upstreamFrames.length}`,
    `rust_job_frame_count: ${rustFrames.length}`,
    "",
    "## Verdict",
    "",
  ];

  if (blockedReasons.length > 0) {
    lines.push(`blocked_reasons: ${blockedReasons.join(",")}`, "");
    lines.push("The comparator could not produce an upstream-vs-Rust field verdict.");
    lines.push("No Rust job-field patch is justified from this comparator run.");
    return `${lines.join("\n")}\n`;
  }

  if (selectedPair) {
    lines.push("comparison_pair: best_field_match");
    lines.push(`upstream_frame_index: ${selectedPair.upstreamIndex}`);
    lines.push(`rust_frame_index: ${selectedPair.rustIndex}`);
    lines.push(
      `matched_field_count: ${selectedPair.matchedFieldCount}/${selectedPair.comparedFieldCount}`,
    );
    lines.push("");
  }

  lines.push(`mismatched_fields: ${mismatches.length === 0 ? "none" : mismatches.join(",")}`, "");
  lines.push("| Field | Verdict |");
  lines.push("| --- | --- |");
  for (const row of rows) {
    lines.push(`| ${row.field} | ${row.verdict} |`);
  }
  lines.push("");

  if (status === "match") {
    lines.push("The compared first job-frame fields match. This does not prove hashing.");
  } else if (status === "mismatch_unconfirmed_dynamic_work_fields") {
    lines.push(
      "Only dynamic source-work fields differ, and source-work alignment is not proven. No Rust job-field patch is justified from this run.",
    );
  } else {
    lines.push(
      "One or more compared best-pair job-frame fields differ. Patch only one confirmed divergence at a time.",
    );
  }

  return `${lines.join("\n")}\n`;
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (!args.out) {
    usage(1);
  }

  const report = reportFor({
    upstreamPath: args.upstream,
    rustPath: args.rust,
    upstreamSourceWorkPath: args.upstreamSourceWork,
    rustSourceWorkPath: args.rustSourceWork,
    upstreamText: readMaybeFile(args.upstream),
    rustText: readMaybeFile(args.rust),
  });

  fs.mkdirSync(path.dirname(args.out), { recursive: true });
  fs.writeFileSync(args.out, report, "utf8");
}

main();
