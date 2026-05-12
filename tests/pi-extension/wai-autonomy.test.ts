import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  contextInstructionForUsage,
  findWaiRoot,
  hasActivePipeline,
  isCommitCommand,
  isPushCommand,
  isReadOnlyOrWorkflowCommand,
  isVerificationCommand,
  promptExplicitlyAllowsPush,
  recordsVerificationWaiver,
} from "../../extensions/wai-autonomy/index.ts";

test("findWaiRoot finds nearest ancestor workspace", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "wai-autonomy-"));
  const nested = path.join(root, "a", "b", "c");
  fs.mkdirSync(path.join(root, ".wai"), { recursive: true });
  fs.mkdirSync(nested, { recursive: true });

  assert.equal(findWaiRoot(nested), root);
  assert.equal(findWaiRoot(root), root);
});

test("findWaiRoot returns undefined outside wai workspace", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "wai-autonomy-none-"));
  fs.mkdirSync(path.join(root, "nested"), { recursive: true });

  assert.equal(findWaiRoot(path.join(root, "nested")), undefined);
});

test("hasActivePipeline detects current and last-run markers", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "wai-autonomy-pipeline-"));
  fs.mkdirSync(path.join(root, ".wai"), { recursive: true });
  assert.equal(hasActivePipeline(root), false);

  fs.writeFileSync(path.join(root, ".wai", ".pipeline-run"), "run-1");
  assert.equal(hasActivePipeline(root), true);

  fs.rmSync(path.join(root, ".wai", ".pipeline-run"));
  fs.mkdirSync(path.join(root, ".wai", "resources", "pipelines"), { recursive: true });
  fs.writeFileSync(path.join(root, ".wai", "resources", "pipelines", ".last-run"), "run-2");
  assert.equal(hasActivePipeline(root), true);
});

test("verification command classifier recognizes common quality gates", () => {
  assert.equal(isVerificationCommand("cargo test -p wai"), true);
  assert.equal(isVerificationCommand("npm run lint"), true);
  assert.equal(isVerificationCommand("just check"), true);
  assert.equal(isVerificationCommand("git status --short"), false);
});

test("workflow classifiers recognize push, commit, and waivers", () => {
  assert.equal(isPushCommand("git push origin main"), true);
  assert.equal(isCommitCommand("git commit -m 'x'"), true);
  assert.equal(recordsVerificationWaiver('wai add research "VERIFICATION NOT RUN: no test harness"'), true);
  assert.equal(recordsVerificationWaiver("wai add research \"notes only\""), false);
});

test("read-only workflow classifier distinguishes mutating bash", () => {
  assert.equal(isReadOnlyOrWorkflowCommand("wai status"), true);
  assert.equal(isReadOnlyOrWorkflowCommand("git diff --stat"), true);
  assert.equal(isReadOnlyOrWorkflowCommand("cargo test -p wai"), false);
  assert.equal(isReadOnlyOrWorkflowCommand("git add src/lib.rs"), false);
});

test("push authorization and context thresholds are explicit", () => {
  assert.equal(promptExplicitlyAllowsPush("push this after tests pass"), true);
  assert.equal(promptExplicitlyAllowsPush("please finish the refactor"), false);

  assert.match(contextInstructionForUsage(35), />=35%/);
  assert.match(contextInstructionForUsage(40), />=40%/);
  assert.equal(contextInstructionForUsage(20), "");
});
