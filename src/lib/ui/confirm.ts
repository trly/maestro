import { writable } from "svelte/store"

export interface ConfirmOptions {
	title: string
	message: string
	confirmText?: string
	cancelText?: string
}

interface ConfirmState extends ConfirmOptions {
	isOpen: boolean
	resolve?: (value: boolean) => void
}

export const confirmState = writable<ConfirmState>({
	isOpen: false,
	title: "",
	message: "",
})

export function confirm(options: ConfirmOptions): Promise<boolean> {
	return new Promise((resolve) => {
		confirmState.set({
			...options,
			isOpen: true,
			resolve,
		})
	})
}

export function confirmYes() {
	confirmState.update((state) => {
		state.resolve?.(true)
		return { ...state, isOpen: false }
	})
}

export function confirmNo() {
	confirmState.update((state) => {
		state.resolve?.(false)
		return { ...state, isOpen: false }
	})
}
