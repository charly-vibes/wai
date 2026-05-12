import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

export function findWaiRoot(cwd: string): string | undefined {
  let current = path.resolve(cwd);
  while (true) {
    if (fs.existsSync(path.join(current, ".wai"))) return current;
    const parent = path.dirname(current);
    if (parent === current) return undefined;
    current = parent;
  }
}

export function hasActivePipeline(root: string): boolean {
  return (
    fs.existsSync(path.join(root, ".wai", ".pipeline-run")) ||
    fs.existsSync(path.join(root, ".wai", "resources", "pipelines", ".last-run"))
  );
}

export function isVerificationCommand(command: string): boolean {
  return /\b(test|pytest|cargo\s+(test|check|clippy|build|nextest)|npm\s+(test|run\s+(test|lint|typecheck|build|ci))|pnpm\s+(test|lint|typecheck|build)|yarn\s+(test|lint|typecheck|build)|bun\s+test|uv\s+run\s+pytest|vitest|jest|go\s+test|just\s+(test|check|lint|build)|make\s+test|clippy|lint|typecheck|build)\b/i.test(
    command,
  );
}

export function isPushCommand(command: string): boolean {
  return /\bgit\s+push\b/.test(command);
}

export function isCommitCommand(command: string): boolean {
  return /\bgit\s+commit\b/.test(command);
}

export function promptExplicitlyAllowsPush(prompt: string): boolean {
  return /\b(push|release|deploy)\b/i.test(prompt);
}

export function recordsVerificationWaiver(command: string): boolean {
  return /\bwai\s+add\s+(research|plan|design)\b[\s\S]*VERIFICATION NOT RUN:/i.test(command);
}

export function contextInstructionForUsage(percent: number | null | undefined): string {
  if (percent == null) return "";
  if (percent >= 40) {
    return "\n- Context usage is >=40%. Do not continue implementation. Create/update wai handoff and stop.\n";
  }
  if (percent >= 35) {
    return "\n- Context usage is >=35%. Prefer finishing the current atomic step, then run wai close.\n";
  }
  return "";
}

export function isReadOnlyOrWorkflowCommand(command: string): boolean {
  const trimmed = command.trim();
  return /^(ls|pwd|rg|grep|find|fd|tree|git\s+(status|diff|log|show|branch)|bd\s+(ready|show|list|status|count|graph)|wai\s+(prime|status|search|pipeline\s+(list|show|gates|check|validate|status)|why)|openspec\s+(list|show|validate))\b/i.test(
    trimmed,
  );
}

function gitHasPendingChanges(cwd: string): boolean {
  try {
    const output = execFileSync("git", ["status", "--porcelain"], {
      cwd,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
    return output.trim().length > 0;
  } catch {
    return false;
  }
}

function autonomousPolicy(waiRoot: string, activePipeline: boolean, contextInstruction: string): string {
  return `

WAI AUTONOMOUS WORK POLICY:
- This is a wai workspace at ${waiRoot}.
- When the user says work/continue/next or context is unclear, orient with wai prime/status before editing.
- ${activePipeline ? "An active wai pipeline appears to exist; follow its current step and gates." : "If the task is implementation work, prefer the repo's wai pipeline when available."}
- Do not ask routine confirmations for continue/fix/commit when the next step is clear.
- Ask only for conflicting requirements, destructive actions, credentials/external services, unresolved test failures, or push/deploy/release.
- Before final answer include Changed / Verified / Risks / Next.
- If code changed, run relevant tests/checks or explicitly record why verification was not possible with: wai add research "VERIFICATION NOT RUN: <reason>".${contextInstruction}`;
}

export default function (pi: ExtensionAPI) {
  let verificationFresh = false;
  let verificationWaived = false;
  let explicitPushAllowed = false;

  pi.on("before_agent_start", async (event, ctx) => {
    const waiRoot = findWaiRoot(ctx.cwd);
    if (!waiRoot) return;

    explicitPushAllowed = promptExplicitlyAllowsPush(event.prompt);

    const usage = ctx.getContextUsage();
    const activePipeline = hasActivePipeline(waiRoot);
    const contextInstruction = contextInstructionForUsage(usage?.percent);

    return {
      systemPrompt: event.systemPrompt + autonomousPolicy(waiRoot, activePipeline, contextInstruction),
    };
  });

  pi.on("tool_call", async (event, ctx) => {
    if (event.toolName === "edit" || event.toolName === "write") {
      verificationFresh = false;
      verificationWaived = false;
      return;
    }

    if (event.toolName !== "bash") return;
    const command = String((event.input as { command?: unknown }).command ?? "");

    if (isVerificationCommand(command)) {
      verificationFresh = true;
      verificationWaived = false;
      return;
    }

    if (recordsVerificationWaiver(command)) {
      verificationWaived = true;
      return;
    }

    if (isCommitCommand(command)) {
      if (gitHasPendingChanges(ctx.cwd) && !verificationFresh && !verificationWaived) {
        return {
          block: true,
          reason:
            "Blocked by wai-autonomy: git has pending changes but no fresh verification command was observed. Run relevant tests/checks or record `wai add research \"VERIFICATION NOT RUN: <reason>\"` before committing.",
        };
      }
      return;
    }

    if (!explicitPushAllowed && isPushCommand(command)) {
      return {
        block: true,
        reason:
          "Blocked by wai-autonomy: git push requires explicit user authorization in the current prompt.",
      };
    }

    if (!isReadOnlyOrWorkflowCommand(command)) {
      verificationFresh = false;
      verificationWaived = false;
    }
  });

  pi.on("agent_end", async (_event, ctx) => {
    const waiRoot = findWaiRoot(ctx.cwd);
    if (!waiRoot) return;

    const usage = ctx.getContextUsage();
    if (ctx.hasUI && usage?.percent != null && usage.percent >= 35) {
      ctx.ui.notify(
        `wai-autonomy: context ${usage.percent.toFixed(1)}%; prefer wai close before more implementation.`,
        "warning",
      );
    }
  });

  pi.on("session_before_compact", async (event, ctx) => {
    const waiRoot = findWaiRoot(ctx.cwd);
    if (!waiRoot) return;

    return {
      compaction: {
        summary:
          "Wai workspace compaction. Preserve: current task, active pipeline step, files changed, verification commands/results, review findings, risks, and exact next action. Prefer wai close plus a fresh session when possible.",
        firstKeptEntryId: event.preparation.firstKeptEntryId,
        tokensBefore: event.preparation.tokensBefore,
      },
    };
  });
}
