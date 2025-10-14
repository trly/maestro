import { writable } from 'svelte/store';
import * as ipc from '$lib/ipc';

export interface Settings {
	ciStuckThresholdMinutes: number;
	editorCommand: string;
	selectedEditor: string;
	selectedTerminal: string;
}

const defaultSettings: Settings = {
	ciStuckThresholdMinutes: 10,
	editorCommand: 'code',
	selectedEditor: 'code',
	selectedTerminal: ''
};

function createSettingsStore() {
	const { subscribe, set, update } = writable<Settings>(defaultSettings);

	return {
		subscribe,
		async load() {
			const threshold = await ipc.getCiStuckThresholdMinutes();
			const editorCmd = await ipc.getSetting('editor_command');
			const selectedEditor = await ipc.getSetting('selected_editor');
			const selectedTerminal = await ipc.getSetting('selected_terminal');
			update(s => ({ 
				...s, 
				ciStuckThresholdMinutes: threshold,
				editorCommand: editorCmd || defaultSettings.editorCommand,
				selectedEditor: selectedEditor || defaultSettings.selectedEditor,
				selectedTerminal: selectedTerminal || defaultSettings.selectedTerminal
			}));
		},
		async setCiStuckThreshold(minutes: number) {
			await ipc.setSetting('ci_stuck_threshold_minutes', minutes.toString());
			update(s => ({ ...s, ciStuckThresholdMinutes: minutes }));
		},
		async setEditorCommand(command: string) {
			await ipc.setSetting('editor_command', command);
			update(s => ({ ...s, editorCommand: command }));
		},
		async setSelectedEditor(editor: string) {
			await ipc.setSetting('selected_editor', editor);
			update(s => ({ ...s, selectedEditor: editor }));
		},
		async setSelectedTerminal(terminal: string) {
			await ipc.setSetting('selected_terminal', terminal);
			update(s => ({ ...s, selectedTerminal: terminal }));
		}
	};
}

export const settingsStore = createSettingsStore();
