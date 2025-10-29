import { writable } from "svelte/store"
import * as ipc from "$lib/ipc"

export interface Settings {
	ciStuckThresholdMinutes: number
	maxConcurrentExecutions: number
	editorCommand: string
	selectedEditor: string
	selectedTerminal: string
	ui?: {
		sidebarPct: number
		promptPct: number
		promptSplitPct: number
		diffFilesPct: number
	}
}

const defaultSettings: Settings = {
	ciStuckThresholdMinutes: 10,
	maxConcurrentExecutions: 10,
	editorCommand: "code",
	selectedEditor: "code",
	selectedTerminal: "",
	ui: {
		sidebarPct: 20,
		promptPct: 35,
		promptSplitPct: 50,
		diffFilesPct: 25,
	},
}

function createSettingsStore() {
	const { subscribe, set, update } = writable<Settings>(defaultSettings)

	return {
		subscribe,
		async load() {
			const threshold = await ipc.getCiStuckThresholdMinutes()
			const maxConcurrent = await ipc.getMaxConcurrentExecutions()
			const editorCmd = await ipc.getSetting("editor_command")
			const selectedEditor = await ipc.getSetting("selected_editor")
			const selectedTerminal = await ipc.getSetting("selected_terminal")
			const uiSettingsStr = await ipc.getSetting("ui_settings")
			const uiSettings = uiSettingsStr ? JSON.parse(uiSettingsStr) : defaultSettings.ui
			update((s) => ({
				...s,
				ciStuckThresholdMinutes: threshold,
				maxConcurrentExecutions: maxConcurrent,
				editorCommand: editorCmd || defaultSettings.editorCommand,
				selectedEditor: selectedEditor || defaultSettings.selectedEditor,
				selectedTerminal: selectedTerminal || defaultSettings.selectedTerminal,
				ui: uiSettings,
			}))
		},
		async setCiStuckThreshold(minutes: number) {
			await ipc.setSetting("ci_stuck_threshold_minutes", minutes.toString())
			update((s) => ({ ...s, ciStuckThresholdMinutes: minutes }))
		},
		async setMaxConcurrentExecutions(count: number) {
			await ipc.setSetting("max_concurrent_executions", count.toString())
			update((s) => ({ ...s, maxConcurrentExecutions: count }))
		},
		async setEditorCommand(command: string) {
			await ipc.setSetting("editor_command", command)
			update((s) => ({ ...s, editorCommand: command }))
		},
		async setSelectedEditor(editor: string) {
			await ipc.setSetting("selected_editor", editor)
			update((s) => ({ ...s, selectedEditor: editor }))
		},
		async setSelectedTerminal(terminal: string) {
			await ipc.setSetting("selected_terminal", terminal)
			update((s) => ({ ...s, selectedTerminal: terminal }))
		},
		async updateUI(uiSettings: Partial<Settings["ui"]>) {
			let mergedUI: Settings["ui"]
			update((s) => {
				mergedUI = { ...defaultSettings.ui!, ...s.ui, ...uiSettings }
				return { ...s, ui: mergedUI }
			})
			await ipc.setSetting("ui_settings", JSON.stringify(mergedUI))
		},
	}
}

export const settingsStore = createSettingsStore()
