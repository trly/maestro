import { tauriApi } from "./tauri-api"
import type { Repository, PromptSet, PromptRevision, Execution } from "./types"

// Since we've migrated to Tauri-only, simply export the Tauri API
export const api = tauriApi

// Type re-export for convenience
export type { Repository, PromptSet, PromptRevision, Execution }
