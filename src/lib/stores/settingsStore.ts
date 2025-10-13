import { writable } from 'svelte/store';
import * as ipc from '$lib/ipc';

export interface Settings {
	ciStuckThresholdMinutes: number;
	editorCommand: string;
}

const defaultSettings: Settings = {
	ciStuckThresholdMinutes: 10,
	editorCommand: 'code'
};

function createSettingsStore() {
	const { subscribe, set, update } = writable<Settings>(defaultSettings);

	return {
		subscribe,
		async load() {
			const threshold = await ipc.getCiStuckThresholdMinutes();
			const editorCmd = await ipc.getSetting('editor_command');
			update(s => ({ 
				...s, 
				ciStuckThresholdMinutes: threshold,
				editorCommand: editorCmd || defaultSettings.editorCommand
			}));
		},
		async setCiStuckThreshold(minutes: number) {
			await ipc.setSetting('ci_stuck_threshold_minutes', minutes.toString());
			update(s => ({ ...s, ciStuckThresholdMinutes: minutes }));
		},
		async setEditorCommand(command: string) {
			await ipc.setSetting('editor_command', command);
			update(s => ({ ...s, editorCommand: command }));
		}
	};
}

export const settingsStore = createSettingsStore();
