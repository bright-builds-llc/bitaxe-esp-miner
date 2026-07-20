#!/usr/bin/env bash
set -euo pipefail

runfiles_root="${TEST_SRCDIR:-}"
if [[ -n "$runfiles_root" && -f "${runfiles_root}/_main/scripts/verify-redaction.sh" ]]; then
	source_script="${runfiles_root}/_main/scripts/verify-redaction.sh"
	source_registry="${runfiles_root}/_main/scripts/redaction-exceptions.tsv"
else
	script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
	source_script="${script_dir}/verify-redaction.sh"
	source_registry="${script_dir}/redaction-exceptions.tsv"
fi

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/verify-redaction-test.XXXXXX")"
trap 'rm -rf "$tmp_root"' EXIT

new_repo() {
	local name="$1"
	local repo="${tmp_root}/${name}"

	mkdir -p "${repo}/scripts" "${repo}/docs/parity/evidence"
	cp "$source_script" "${repo}/scripts/verify-redaction.sh"
	cp "$source_registry" "${repo}/scripts/redaction-exceptions.tsv"
	chmod 755 "${repo}/scripts/verify-redaction.sh"
	git -C "$repo" init -q
	git -C "$repo" config user.email evidence-test@example.invalid
	git -C "$repo" config user.name evidence-test
	printf 'safe\n' >"${repo}/README.md"
	git -C "$repo" add README.md scripts
	git -C "$repo" commit -qm seed
	printf '%s\n' "$repo"
}

assert_fails_without_echo() {
	local repo="$1"
	local forbidden="$2"
	shift 2
	local output

	set +e
	output="$(cd "$repo" && "$@" 2>&1)"
	status=$?
	set -e
	[[ "$status" -eq 1 ]]
	[[ "$output" == *"redaction_violation: rule="* ]]
	[[ "$output" != *"$forbidden"* ]]
}

repo="$(new_repo staged)"
staged_secret="fixture-stage-secret"
printf '%s%s\n' 'pass' "word=${staged_secret}" >"${repo}/staged.txt"
git -C "$repo" add staged.txt
assert_fails_without_echo "$repo" "$staged_secret" bash ./scripts/verify-redaction.sh

repo="$(new_repo ci)"
base_commit="$(git -C "$repo" rev-parse HEAD)"
ci_secret="fixture-ci-secret"
printf '%s%s\n' 'to' "ken=${ci_secret}" >"${repo}/ci.txt"
git -C "$repo" add ci.txt
git -C "$repo" commit -qm ci
head_commit="$(git -C "$repo" rev-parse HEAD)"
assert_fails_without_echo "$repo" "$ci_secret" \
	bash ./scripts/verify-redaction.sh --base "$base_commit" --head "$head_commit"

repo="$(new_repo rename)"
rename_secret="fixture-rename-secret"
printf '%s%s\n' 'secret' "=${rename_secret}" >"${repo}/before.txt"
git -C "$repo" add before.txt
git -C "$repo" commit -qm fixture
git -C "$repo" mv before.txt after.txt
assert_fails_without_echo "$repo" "$rename_secret" bash ./scripts/verify-redaction.sh

repo="$(new_repo exception)"
exception_secret="fixture-reviewed-secret"
printf '%s%s\n' 'pass' "word=${exception_secret}" >"${repo}/docs/parity/evidence/reviewed.txt"
printf 'RED-9001\tcredential-secret\tdocs/parity/evidence/reviewed.txt\tsynthetic reviewed test fixture\t2099-01-01\n' \
	>>"${repo}/scripts/redaction-exceptions.tsv"
git -C "$repo" add docs/parity/evidence/reviewed.txt scripts/redaction-exceptions.tsv
git -C "$repo" commit -qm reviewed-baseline
(cd "$repo" && bash ./scripts/verify-redaction.sh >/dev/null)
changed_exception_secret="fixture-changed-reviewed-secret"
printf '%s%s\n' 'pass' "word=${changed_exception_secret}" >"${repo}/docs/parity/evidence/reviewed.txt"
git -C "$repo" add docs/parity/evidence/reviewed.txt
assert_fails_without_echo "$repo" "$changed_exception_secret" bash ./scripts/verify-redaction.sh

repo="$(new_repo source-operational)"
mkdir -p "${repo}/src"
printf 'const FIXTURE: &str = "/Users/example/device 192.0.2.80 /dev/cu.example";\n' \
	>"${repo}/src/operational_fixture.rs"
git -C "$repo" add src/operational_fixture.rs
(cd "$repo" && bash ./scripts/verify-redaction.sh >/dev/null)

repo="$(new_repo shareable-operational)"
shareable_origin="192.0.2.81"
printf 'observed_address=%s\n' "$shareable_origin" >"${repo}/docs/shareable.md"
git -C "$repo" add docs/shareable.md
assert_fails_without_echo "$repo" "$shareable_origin" bash ./scripts/verify-redaction.sh

repo="$(new_repo admitted)"
admitted_origin="192.0.2.44"
printf 'origin=%s\n' "$admitted_origin" >"${repo}/docs/parity/evidence/leak.md"
assert_fails_without_echo "$repo" "$admitted_origin" bash ./scripts/verify-redaction.sh

repo="$(new_repo malformed)"
printf 'invalid\n' >"${repo}/scripts/redaction-exceptions.tsv"
git -C "$repo" add scripts/redaction-exceptions.tsv
set +e
malformed_output="$(cd "$repo" && bash ./scripts/verify-redaction.sh 2>&1)"
malformed_status=$?
set -e
[[ "$malformed_status" -eq 2 ]]
[[ "$malformed_output" == *"rule=CONFIG category=exception-registry"* ]]

repo="$(new_repo new-branch)"
new_branch_secret="fixture-new-branch-secret"
printf '%s%s\n' 'to' "ken=${new_branch_secret}" >"${repo}/already-at-head.txt"
git -C "$repo" add already-at-head.txt
git -C "$repo" commit -qm new-branch-head
new_branch_head="$(git -C "$repo" rev-parse HEAD)"
assert_fails_without_echo "$repo" "$new_branch_secret" bash ./scripts/verify-redaction.sh \
	--base 0000000000000000000000000000000000000000 --head "$new_branch_head"

repo="$(new_repo bad-revision)"
bad_head="$(git -C "$repo" rev-parse HEAD)"
set +e
bad_revision_output="$(cd "$repo" && bash ./scripts/verify-redaction.sh \
	--base deadbeefdeadbeefdeadbeefdeadbeefdeadbeef --head "$bad_head" 2>&1)"
bad_revision_status=$?
set -e
[[ "$bad_revision_status" -eq 2 ]]
[[ "$bad_revision_output" == *"rule=CONFIG category=revision"* ]]

repo="$(new_repo safe)"
printf 'status_class=completed count=2 duration_seconds=10\n' >"${repo}/safe.txt"
git -C "$repo" add safe.txt
(cd "$repo" && bash ./scripts/verify-redaction.sh >/dev/null)

printf 'verify_redaction_test: passed\n'
