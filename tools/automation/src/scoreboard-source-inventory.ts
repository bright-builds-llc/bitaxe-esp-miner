import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";

export const scoreboardSourceFragments = new Map<string, readonly string[]>([
  ["crates/bitaxe-api/src/scoreboard.rs", ["pub fn scoreboard_response(entries: &[ScoreboardEntry])"]],
  ["crates/bitaxe-api/src/scoreboard/owner.rs", [
    "pub const MAX_SCOREBOARD_ENTRIES: usize = 20;",
    "\"{:.1};{};{};{};{};{}\",",
  ]],
  ["crates/bitaxe-stratum/src/v1/production_session/runtime/asic.rs", ["ProductionSessionEffect::RecordScoreboard"]],
  ["crates/bitaxe-automation-contracts/src/scoreboard_evidence.rs", ["pub struct ScoreboardEvidence {"]],
  ["crates/bitaxe-automation-contracts/src/bin/validate_scoreboard_evidence.rs", ["let evidence: ScoreboardEvidence ="]],
  ["firmware/bitaxe/src/production_mining_session/scoreboard.rs", ["pub(super) fn record(candidate: ScoreboardCandidate) -> Option<ProductionSessionEvent>"]],
  ["firmware/bitaxe/src/scoreboard_adapter.rs", ["pub fn record_candidate(candidate: ScoreboardCandidate)"]],
  ["firmware/bitaxe/src/runtime_snapshot.rs", ["pub fn projected_scoreboard(_timestamp_ms: u64) -> Vec<ScoreboardEntryWire>"]],
  ["firmware/bitaxe/src/http_api/handlers.rs", ["pub(super) fn handle_scoreboard<'request, 'connection>("]],
  ["crates/bitaxe-api/src/static_plan.rs", ["const OPERATOR_UI_ROUTES: &[&str] = &["]],
  ["tools/flash/src/campaign/admission.rs", ["pub(super) fn campaign_nvs_csv("]],
  ["tools/flash/src/campaign/network/terminal_settlement.rs", ["pub(super) const fn terminal_settlement("]],
  ["tools/flash/src/campaign/network/observer.rs", ["TerminalSettlementDecision::RequestSerialClose => request_serial_close(&shared),"]],
  ["tools/flash/src/campaign/network/model.rs", ["terminal_settlement: self.terminal_settlement,"]],
  ["tools/flash/src/campaign/network/model/evidence.rs", ["pub(in crate::campaign) final_terminal_consumed: bool,"]],
  ["firmware/bitaxe/src/startup.rs", ["scoreboard_adapter::initialize()"]],
  ["firmware/bitaxe/static/www/index.html", ["data-page=\"scoreboard\""]],
  ["firmware/bitaxe/static/www/assets/ui-core.js", ["function scoreboardRows(payload)"]],
  ["firmware/bitaxe/static/www/assets/api-client.js", ["async function getScoreboard()"]],
  ["firmware/bitaxe/static/www/assets/app.js", ["async function refreshScoreboard()"]],
  ["firmware/bitaxe/static/www/assets/app.css", [".scoreboard-card table"]],
  ["tools/automation/src/scoreboard-evidence.ts", ["export async function captureScoreboardEvidence("]],
  ["tools/automation/src/scoreboard-evidence-contract.ts", ["export async function validateScoreboardTaskAndSources("]],
  ["tools/automation/src/scoreboard-source-inventory.ts", []],
  ["tools/automation/src/http.ts", ["export async function fetchJsonArrayFromSameOrigin("]],
  ["tools/automation/src/cli.ts", ["case \"capture-scoreboard-evidence\":"]],
]);

export const scoreboardReferenceFragments = new Map<string, readonly string[]>([
  ["reference/esp-miner/main/tasks/scoreboard.c", [
    "esp_err_t scoreboard_add(Scoreboard *scoreboard",
    "sscanf(entry_str, \"%lf;%31[^;];%31[^;];%lu;%lu;%lu\",",
    "\"%.1f;%s;%s;%lu;%lu;%lu\",",
  ]],
  ["reference/esp-miner/main/tasks/asic_result_task.c", ["scoreboard_add(&GLOBAL_STATE->SYSTEM_MODULE.scoreboard"]],
  ["reference/esp-miner/main/http_server/http_server.c", ["static esp_err_t GET_scoreboard(httpd_req_t * req)"]],
  ["reference/esp-miner/main/http_server/axe-os/src/app/components/scoreboard/scoreboard.component.ts", ["export class ScoreboardComponent"]],
  ["reference/esp-miner/main/http_server/axe-os/src/app/services/system.service.ts", ["public getScoreboard(uri: string = '')"]],
]);

export async function scoreboardSourceInventory(workspaceRoot: string): Promise<{
  readonly digest: string;
  readonly pathCount: number;
}> {
  const inventory = [...scoreboardSourceFragments, ...scoreboardReferenceFragments];
  const digest = createHash("sha256");
  for (const [relative, fragments] of inventory) {
    const document = await readFile(path.join(workspaceRoot, relative));
    const text = document.toString("utf8");
    for (const fragment of fragments) {
      if (text.split(fragment).length !== 2) {
        throw new Error("scoreboard source semantics are invalid");
      }
    }
    digest.update(relative).update("\0").update(document).update("\0");
  }
  return { digest: digest.digest("hex"), pathCount: inventory.length };
}
