import { execute } from "@sourcegraph/amp-sdk"

async function executeWithAmp(
	repoPath: string,
	promptText: string,
	continueSessionId?: string,
	onSessionStart?: (sessionId: string) => void
): Promise<{ sessionId: string; resultMessage?: string }> {
	let sessionId = continueSessionId || ""
	let resultMessage: string | undefined

	const abortController = new AbortController()

	// Handle SIGTERM to cancel execution gracefully
	const sigHandler = () => {
		console.error(JSON.stringify({ type: "abort", message: "Received termination signal" }))
		abortController.abort()
	}
	process.on("SIGTERM", sigHandler)
	process.on("SIGINT", sigHandler)

	try {
		for await (const message of execute({
			prompt: promptText,
			options: {
				dangerouslyAllowAll: true,
				cwd: repoPath,
				...(continueSessionId ? { continue: continueSessionId } : {}),
			},
			signal: abortController.signal,
		})) {
			if (message.type === "system" && message.subtype === "init") {
				sessionId = message.session_id
				if (onSessionStart) {
					onSessionStart(sessionId)
				}
				console.error(JSON.stringify({ type: "session_id", sessionId: message.session_id }))
			}
			if (message.type === "result" && !message.is_error) {
				resultMessage = (message as any).result
			}
		}
	} finally {
		process.off("SIGTERM", sigHandler)
		process.off("SIGINT", sigHandler)
	}

	return { sessionId, resultMessage }
}

const cwd = process.argv[2]
const prompt = process.argv[3]
const continueSessionId = process.argv[4]

if (!cwd || !prompt) {
	process.exit(1)
}

executeWithAmp(cwd, prompt, continueSessionId)
	.then((result) => {
		console.log(JSON.stringify(result))
		process.exit(0)
	})
	.catch((error) => {
		console.error(JSON.stringify({ error: String(error) }))
		process.exit(1)
	})
