import {
  execute,
  type StreamMessage,
  type ResultMessage,
} from "@sourcegraph/amp-sdk";
import { Store } from "./db/store";
import type { Execution, Repository, PromptRevision } from "./types";

const GITHUB_TOKEN = process.env.VITE_MAESTRO_GITHUB_TOKEN;
const CLONE_DIR =
  process.env.VITE_MAESTRO_CLONE_DIR || `${process.env.HOME}/maestro/repos`;

interface ExecuteOptions {
  executionId: string;
  store: Store;
}

interface ExecutePromptSetOptions {
  promptsetId: string;
  revisionId: string;
  store: Store;
}

export async function executePromptSet({
  promptsetId,
  revisionId,
  store,
}: ExecutePromptSetOptions): Promise<string[]> {
  const promptSet = store.getPromptSet(promptsetId);
  if (!promptSet) {
    throw new Error(`PromptSet ${promptsetId} not found`);
  }

  const executionIds: string[] = [];

  for (const repositoryId of promptSet.repositoryIds) {
    const execution = store.createExecution(
      promptsetId,
      revisionId,
      repositoryId,
    );
    executionIds.push(execution.id);

    executePrompt({ executionId: execution.id, store }).catch((err) =>
      console.error(`Execution ${execution.id} failed:`, err),
    );
  }

  return executionIds;
}

export async function executePrompt({
  executionId,
  store,
}: ExecuteOptions): Promise<void> {
  console.log(`[executePrompt] Starting execution ${executionId}`);

  const execution = store.getExecution(executionId);
  if (!execution) {
    throw new Error(`Execution ${executionId} not found`);
  }

  const repository = store.getRepository(execution.repositoryId);
  if (!repository) {
    throw new Error(`Repository ${execution.repositoryId} not found`);
  }

  const revision = store.getPromptRevision(execution.revisionId);
  if (!revision) {
    throw new Error(`Revision ${execution.revisionId} not found`);
  }

  console.log(`[executePrompt] Updating status to running for ${executionId}`);
  store.updateExecution(executionId, { 
    status: "running",
    filesAdded: 0,
    filesRemoved: 0,
    filesModified: 0,
    linesAdded: 0,
    linesRemoved: 0,
  });

  try {
    console.log(`[executePrompt] Cloning repository ${repository.providerId}`);
    const repoPath = await cloneRepository(repository, execution, revision);
    console.log(`[executePrompt] Repository cloned to ${repoPath}`);

    console.log(`[executePrompt] Creating branch for ${executionId}`);
    const baseCommit = await createBranch(
      repoPath,
      execution.promptsetId,
      execution.revisionId,
      executionId,
    );

    console.log(`[executePrompt] Executing with Amp for ${executionId}`);
    const responseFormat = `

IMPORTANT: You MUST end your final response with exactly one of these lines on the final line to reflect if the above prompt is considered successful or not:
PROMPT: PASS
PROMPT: FAIL`;
	const fullPrompt = revision.promptText + responseFormat;
    const { sessionId, resultMessage } = await executeWithAmp(repoPath, fullPrompt);
    const threadUrl = `https://ampcode.com/threads/${sessionId}`;

    const promptPassed = resultMessage?.includes("PROMPT: PASS") ?? false;
    const promptFailed = resultMessage?.includes("PROMPT: FAIL") ?? false;
    const promptStatus = promptPassed ? "passed" : promptFailed ? "failed" : null;

    console.log(`[executePrompt] Capturing diff statistics for ${executionId}`);
	const diffStats = await getDiffStats(repoPath, baseCommit);

	console.log(`[executePrompt] Execution completed: ${threadUrl} - Prompt Status: ${promptStatus}`);
	store.updateExecution(executionId, {
		status: "completed",
		sessionId,
		threadUrl,
		promptStatus,
		promptResult: resultMessage || null,
		completedAt: Date.now(),
		...diffStats,
	});

    const promptSet = store.getPromptSet(execution.promptsetId);
    if (promptSet?.validationPrompt && promptStatus === "passed") {
    console.log(`[executePrompt] Running validation for ${executionId}`);
    validateExecution({ executionId, store }).catch((err) =>
    console.error(`Validation ${executionId} failed:`, err),
    );
    } else if (promptSet?.validationPrompt && promptStatus !== "passed") {
		console.log(`[executePrompt] Skipping validation for ${executionId} - prompt status: ${promptStatus}`);
	}
  } catch (error) {
    console.error(`[executePrompt] Execution ${executionId} failed:`, error);
    store.updateExecution(executionId, {
      status: "failed",
      completedAt: Date.now(),
    });
    throw error;
  }
}

async function cloneRepository(
  repository: Repository,
  execution: Execution,
  revision: PromptRevision,
): Promise<string> {
  if (repository.provider !== "github") {
    throw new Error("Only GitHub repositories are supported");
  }

  const [owner, repo] = repository.providerId.split("/");
  const repoPath = `${CLONE_DIR}/${owner}/${repo}`;

  const checkProc = Bun.spawn(["test", "-d", repoPath], {
    stdout: "pipe",
    stderr: "pipe",
  });
  await checkProc.exited;
  const dirExists = checkProc.exitCode === 0;

  if (dirExists) {
    const gitCheckProc = Bun.spawn(["test", "-d", `${repoPath}/.git`], {
      stdout: "pipe",
      stderr: "pipe",
    });
    await gitCheckProc.exited;
    const isGitRepo = gitCheckProc.exitCode === 0;

    if (isGitRepo) {
      const pullProc = Bun.spawn(["git", "pull"], {
        cwd: repoPath,
        stdout: "pipe",
        stderr: "pipe",
      });

      await pullProc.exited;

      if (pullProc.exitCode !== 0) {
        const error = await new Response(pullProc.stderr).text();
        console.warn(`Failed to pull repository: ${error}`);
      }

      return repoPath;
    } else {
      console.log(`[cloneRepository] Directory exists but no .git found, removing: ${repoPath}`);
      const rmProc = Bun.spawn(["rm", "-rf", repoPath], {
        stdout: "pipe",
        stderr: "pipe",
      });
      await rmProc.exited;
    }
  }

  const mkdirProc = Bun.spawn(["mkdir", "-p", `${CLONE_DIR}/${owner}`], {
    stdout: "pipe",
    stderr: "pipe",
  });
  await mkdirProc.exited;

  const cloneUrl = `https://${GITHUB_TOKEN}@github.com/${owner}/${repo}.git`;

  const proc = Bun.spawn(["git", "clone", cloneUrl, repoPath], {
    stdout: "pipe",
    stderr: "pipe",
  });

  await proc.exited;

  if (proc.exitCode !== 0) {
    const error = await new Response(proc.stderr).text();
    throw new Error(`Failed to clone repository: ${error}`);
  }

  return repoPath;
}

async function createBranch(
  repoPath: string,
  promptsetId: string,
  revisionId: string,
  executionId: string,
): Promise<string> {
  const branchName = `maestro/${promptsetId.slice(0, 8)}/${revisionId.slice(0, 8)}/${executionId.slice(0, 8)}`;

  const getBaseCommitProc = Bun.spawn(["git", "rev-parse", "HEAD"], {
    cwd: repoPath,
    stdout: "pipe",
    stderr: "pipe",
  });
  await getBaseCommitProc.exited;
  const baseCommit = (await new Response(getBaseCommitProc.stdout).text()).trim();

  const proc = Bun.spawn(["git", "checkout", "-b", branchName], {
    cwd: repoPath,
    stdout: "pipe",
    stderr: "pipe",
  });

  await proc.exited;

  if (proc.exitCode !== 0) {
    const error = await new Response(proc.stderr).text();
    throw new Error(`Failed to create branch: ${error}`);
  }

  return baseCommit;
}

async function executeWithAmp(
  repoPath: string,
  promptText: string,
  continueSessionId?: string,
): Promise<{ sessionId: string; resultMessage?: string }> {
  let sessionId = continueSessionId || "";
  let resultMessage: string | undefined;

  for await (const message of execute({
    prompt: promptText,
    options: {
      dangerouslyAllowAll: true,
      cwd: repoPath,
      ...(continueSessionId ? { continue: continueSessionId } : {}),
    },
  })) {
    if (message.type === "system" && message.subtype === "init") {
      sessionId = message.session_id;
    }
    if (message.type === "result" && !message.is_error) {
      resultMessage = message.result;
    }
  }

  return { sessionId, resultMessage };
}

async function getDiffStats(repoPath: string, baseCommit?: string): Promise<{
  filesAdded: number;
  filesRemoved: number;
  filesModified: number;
  linesAdded: number;
  linesRemoved: number;
}> {
  const diffTarget = baseCommit || "HEAD";
  const base = ["diff", "--no-ext-diff", "--no-color", "-M", "--diff-filter=AMDR", diffTarget];

  const nsProc = Bun.spawn(["git", ...base, "--name-status", "-z"], { cwd: repoPath, stdout: "pipe", stderr: "pipe" });
  const numProc = Bun.spawn(["git", ...base, "--numstat", "-z"], { cwd: repoPath, stdout: "pipe", stderr: "pipe" });

  await Promise.all([nsProc.exited, numProc.exited]);
  if (nsProc.exitCode !== 0 || numProc.exitCode !== 0) {
    console.warn("Failed to get diff stats, using zeros");
    return { filesAdded: 0, filesRemoved: 0, filesModified: 0, linesAdded: 0, linesRemoved: 0 };
  }

  const ns = await new Response(nsProc.stdout).text();
  const num = await new Response(numProc.stdout).text();

  type Entry = { status: "A" | "M" | "D" | "R"; oldPath?: string; newPath?: string };
  const statusEntries: Entry[] = [];
  const renameMap = new Map<string, string>();
  {
    const parts = ns.split("\0").filter(Boolean);
    for (let i = 0; i < parts.length; ) {
      const tok = parts[i++]!;
      const code = tok[0] as "A" | "M" | "D" | "R";
      if (code === "R") {
        const oldPath = parts[i++] || "";
        const newPath = parts[i++] || "";
        renameMap.set(oldPath, newPath);
        statusEntries.push({ status: "R", oldPath, newPath });
      } else {
        const p = parts[i++] || "";
        if (code === "D") statusEntries.push({ status: "D", oldPath: p });
        else statusEntries.push({ status: code, newPath: p });
      }
    }
  }

  const counts = new Map<string, { added: number; removed: number }>();
  {
    const parts = num.split("\0");
    for (const rec of parts) {
      if (!rec) continue;
      const m = rec.match(/^(\d+|-)\t(\d+|-)\t([\s\S]*)$/);
      if (!m) continue;
      const a = m[1] === "-" ? 0 : parseInt(m[1], 10);
      const r = m[2] === "-" ? 0 : parseInt(m[2], 10);
      const oldPath = m[3];
      counts.set(oldPath, { added: a, removed: r });
      const newPath = renameMap.get(oldPath);
      if (newPath) counts.set(newPath, { added: a, removed: r });
    }
  }

  let filesAdded = 0, filesRemoved = 0, filesModified = 0, linesAdded = 0, linesRemoved = 0;
  for (const e of statusEntries) {
    switch (e.status) {
      case "A": filesAdded++; break;
      case "D": filesRemoved++; break;
      case "M": filesModified++; break;
      case "R": filesModified++; break;
    }
    const key = e.newPath ?? e.oldPath!;
    const c = counts.get(key) ?? (e.oldPath ? counts.get(e.oldPath) : undefined);
    if (c) { linesAdded += c.added; linesRemoved += c.removed; }
  }

  return { filesAdded, filesRemoved, filesModified, linesAdded, linesRemoved };
}

async function deleteBranchForExecution(
  execution: Execution,
  repository: Repository,
): Promise<void> {
  if (repository.provider !== "github") {
    return;
  }

  try {
    const [owner, repo] = repository.providerId.split("/");
    const repoPath = `${CLONE_DIR}/${owner}/${repo}`;
    const branchName = `maestro/${execution.promptsetId.slice(0, 8)}/${execution.revisionId.slice(0, 8)}/${execution.id.slice(0, 8)}`;

    const repoDir = Bun.file(repoPath);
    if (!(await repoDir.exists())) {
      return;
    }

    const proc = Bun.spawn(["/usr/bin/git", "branch", "-D", branchName], {
      cwd: repoPath,
      stdout: "pipe",
      stderr: "pipe",
    });

    await proc.exited;
  } catch (error) {
    console.warn(
      `Failed to delete git branch for execution ${execution.id}:`,
      error,
    );
  }
}

export async function deleteExecution(
  executionId: string,
  store: Store,
): Promise<void> {
  const execution = store.getExecution(executionId);
  if (!execution) {
    throw new Error(`Execution ${executionId} not found`);
  }

  const repository = store.getRepository(execution.repositoryId);
  if (repository) {
    await deleteBranchForExecution(execution, repository);
  }

  store.deleteExecution(executionId);
}

export async function deletePromptSet(
  promptsetId: string,
  store: Store,
): Promise<void> {
  const promptSet = store.getPromptSet(promptsetId);
  if (!promptSet) {
    throw new Error(`PromptSet ${promptsetId} not found`);
  }

  const executions = store.getExecutionsByPromptSet(promptsetId);

  for (const execution of executions) {
    const repository = store.getRepository(execution.repositoryId);
    if (repository) {
      await deleteBranchForExecution(execution, repository);
    }
  }

  store.deletePromptSet(promptsetId);
}

export async function backfillDiffStats(
  executionId: string,
  store: Store,
): Promise<void> {
  const shortId = executionId.slice(0, 8);
  const execution = store.getExecution(executionId);
  if (!execution) {
    throw new Error(`Execution ${executionId} not found`);
  }

  if (execution.status !== "completed") {
    console.log(
      `[backfillDiffStats] Skipping ${shortId} - not completed (status: ${execution.status})`,
    );
    return;
  }

  const hasStats =
    execution.filesAdded > 0 ||
    execution.filesRemoved > 0 ||
    execution.filesModified > 0 ||
    execution.linesAdded > 0 ||
    execution.linesRemoved > 0;

  if (hasStats) {
    console.log(`[backfillDiffStats] Skipping ${shortId} - stats already exist`);
    return;
  }

  const repository = store.getRepository(execution.repositoryId);
  if (!repository) {
    throw new Error(`Repository ${execution.repositoryId} not found`);
  }

  if (repository.provider !== "github") {
    console.log(`[backfillDiffStats] Skipping ${shortId} - unsupported provider`);
    return;
  }

  const [owner, repo] = repository.providerId.split("/");
  const repoPath = `${CLONE_DIR}/${owner}/${repo}`;
  const branchName = `maestro/${execution.promptsetId.slice(0, 8)}/${execution.revisionId.slice(0, 8)}/${executionId.slice(0, 8)}`;

  console.log(`[backfillDiffStats] Checking repoPath: ${repoPath}`);
  const gitDirCheck = Bun.spawn(["test", "-d", `${repoPath}/.git`], {
    stdout: "pipe",
    stderr: "pipe",
  });
  await gitDirCheck.exited;

  if (gitDirCheck.exitCode !== 0) {
    console.log(`[backfillDiffStats] Skipping ${shortId} - repo not cloned at ${repoPath}`);
    return;
  }

  console.log(`[backfillDiffStats] Checking out branch ${branchName}`);
  const checkoutProc = Bun.spawn(["git", "checkout", branchName], {
    cwd: repoPath,
    stdout: "pipe",
    stderr: "pipe",
  });
  await checkoutProc.exited;

  if (checkoutProc.exitCode !== 0) {
    console.log(`[backfillDiffStats] Skipping ${shortId} - branch not found`);
    return;
  }

  console.log(`[backfillDiffStats] Finding base commit for ${shortId}`);
  const getDefaultBranchProc = Bun.spawn(["git", "symbolic-ref", "refs/remotes/origin/HEAD"], {
    cwd: repoPath,
    stdout: "pipe",
    stderr: "pipe",
  });
  await getDefaultBranchProc.exited;
  
  let defaultBranch = "origin/main";
  if (getDefaultBranchProc.exitCode === 0) {
    const output = (await new Response(getDefaultBranchProc.stdout).text()).trim();
    defaultBranch = output.replace("refs/remotes/", "");
  }

  const mergeBaseProc = Bun.spawn(["git", "merge-base", branchName, defaultBranch], {
    cwd: repoPath,
    stdout: "pipe",
    stderr: "pipe",
  });
  await mergeBaseProc.exited;

  if (mergeBaseProc.exitCode !== 0) {
    console.log(`[backfillDiffStats] Skipping ${shortId} - could not find merge base`);
    return;
  }

  const baseCommit = (await new Response(mergeBaseProc.stdout).text()).trim();

  console.log(`[backfillDiffStats] Calculating diff stats for ${shortId} against ${baseCommit.slice(0, 8)}`);
  const diffStats = await getDiffStats(repoPath, baseCommit);

  console.log(
    `[backfillDiffStats] Updating ${shortId} with stats:`,
    diffStats,
  );
  store.updateExecution(executionId, diffStats);
}

export async function validateExecution({
  executionId,
  store,
}: ExecuteOptions): Promise<void> {
  console.log(`[validateExecution] Starting validation ${executionId}`);

  const execution = store.getExecution(executionId);
  if (!execution) {
    throw new Error(`Execution ${executionId} not found`);
  }

  if (execution.status !== "completed") {
    throw new Error(
      `Cannot validate execution ${executionId} - execution must be completed first (current status: ${execution.status})`,
    );
  }

  const promptSet = store.getPromptSet(execution.promptsetId);
  if (!promptSet?.validationPrompt) {
    throw new Error(
      `PromptSet ${execution.promptsetId} has no validation prompt`,
    );
  }

  const repository = store.getRepository(execution.repositoryId);
  if (!repository) {
    throw new Error(`Repository ${execution.repositoryId} not found`);
  }

  console.log(
    `[validateExecution] Updating validation status to running for ${executionId}`,
  );
  store.updateExecution(executionId, { validationStatus: "running" });

  try {
    const [owner, repo] = repository.providerId.split("/");
    const repoPath = `${CLONE_DIR}/${owner}/${repo}`;
    const branchName = `maestro/${execution.promptsetId.slice(0, 8)}/${execution.revisionId.slice(0, 8)}/${executionId.slice(0, 8)}`;

    console.log(`[validateExecution] Checking out branch ${branchName}`);
    const checkoutProc = Bun.spawn(["git", "checkout", branchName], {
      cwd: repoPath,
      stdout: "pipe",
      stderr: "pipe",
    });
    await checkoutProc.exited;

    if (checkoutProc.exitCode !== 0) {
      const error = await new Response(checkoutProc.stderr).text();
      throw new Error(`Failed to checkout branch: ${error}`);
    }

    console.log(
      `[validateExecution] Executing validation with Amp for ${executionId}`,
    );
    const systemPrompt = `You are a code change validation reviewer
You are tasked with ensuring the current changes in ${branchName}.
You are to review the pending changes in the current branch with the oracle, librarian, and any other tools that will not make any further code changes to ensure that the following is true:

`;
    const responseFormat = `

IMPORTANT: You MUST end your response with exactly one of these lines on the final line:
VALIDATION: PASS
VALIDATION: FAIL`;

    const fullValidationPrompt =
      systemPrompt + promptSet.validationPrompt + responseFormat;
    const { sessionId: validationSessionId, resultMessage } =
      await executeWithAmp(repoPath, fullValidationPrompt);
    const validationThreadUrl = `https://ampcode.com/threads/${validationSessionId}`;

    const validationPassed =
      resultMessage?.includes("VALIDATION: PASS") ?? false;
    const validationFailed =
      resultMessage?.includes("VALIDATION: FAIL") ?? false;
    const validationStatus = validationPassed
      ? "passed"
      : validationFailed
        ? "failed"
        : "failed";

    console.log(
      `[validateExecution] Validation completed: ${validationThreadUrl} - Status: ${validationStatus}`,
    );
    store.updateExecution(executionId, {
      validationStatus,
      validationThreadUrl,
      validationResult: resultMessage || null,
    });

    if (validationStatus === "passed" && execution.sessionId) {
      console.log(
        `[validateExecution] Validation passed, resuming thread to commit changes for ${executionId}`,
      );
      commitChanges({ executionId, store, repoPath }).catch((err) =>
        console.error(`Commit ${executionId} failed:`, err),
      );
    }
  } catch (error) {
    console.error(
      `[validateExecution] Validation ${executionId} failed:`,
      error,
    );
    store.updateExecution(executionId, {
      validationStatus: "failed",
    });
    throw error;
  }
}

async function commitChanges({
  executionId,
  store,
  repoPath,
}: {
  executionId: string;
  store: Store;
  repoPath: string;
}): Promise<void> {
  console.log(`[commitChanges] Starting commit for ${executionId}`);

  const execution = store.getExecution(executionId);
  if (!execution?.sessionId) {
    throw new Error(
      `Cannot commit ${executionId} - no session ID found`,
    );
  }

  try {
    const commitPrompt = "Please commit the current changes with an appropriate commit message.";

    console.log(`[commitChanges] Resuming thread ${execution.sessionId} to commit`);
    await executeWithAmp(repoPath, commitPrompt, execution.sessionId);

    console.log(`[commitChanges] Successfully committed changes for ${executionId}`);
  } catch (error) {
    console.error(`[commitChanges] Failed to commit ${executionId}:`, error);
    throw error;
  }
}
