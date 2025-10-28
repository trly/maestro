import { open } from "@tauri-apps/plugin-shell"

export async function openInBrowser(url: string) {
	await open(url)
}
