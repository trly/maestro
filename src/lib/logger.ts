import { trace, debug, info, warn, error, attachConsole } from "@tauri-apps/plugin-log"

export const logger = {
	trace,
	debug,
	info,
	warn,
	error,
}

/**
 * Initialize logging and optionally attach console forwarding
 * Call this once during app initialization
 * Uses globalThis flag to prevent re-initialization during HMR/dev mode
 */
export async function initLogger() {
	// Use global flag to prevent re-initialization during HMR
	if ((globalThis as any).__maestroConsoleAttached) {
		return
	}

	try {
		// Attach console methods to forward to Tauri logging
		await attachConsole()
		;(globalThis as any).__maestroConsoleAttached = true
	} catch (err) {
		console.warn("Failed to attach console to Tauri logging:", err)
	}
}
