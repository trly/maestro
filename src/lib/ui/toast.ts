import { sendNotification } from '@tauri-apps/plugin-notification'

export type ToastType = 'success' | 'error' | 'info'

export async function showToast(message: string, type: ToastType = 'info') {
	await sendNotification({ title: 'Maestro', body: message })
}
