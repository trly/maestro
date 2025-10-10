import { derived } from 'svelte/store';
import type { Execution } from '../types';
import { executionStore } from './executionBus';

export function getExecutionWithUpdates(execution: Execution) {
	return derived(executionStore, ($store) => {
		const updates = $store.get(execution.id);
		if (!updates) return execution;

		return {
			...execution,
			...(updates.sessionId && { sessionId: updates.sessionId }),
			...(updates.threadUrl && { threadUrl: updates.threadUrl }),
			...(updates.status && { status: updates.status }),
			...(updates.validationStatus && { validationStatus: updates.validationStatus }),
			...(updates.validationThreadUrl && { validationThreadUrl: updates.validationThreadUrl }),
			...(updates.commitStatus && { commitStatus: updates.commitStatus }),
			...(updates.commitSha && { commitSha: updates.commitSha }),
			...(updates.committedAt && { committedAt: updates.committedAt }),
		};
	});
}
