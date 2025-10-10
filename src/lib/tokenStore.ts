import * as ipc from './ipc'

export type TokenKey = ipc.TokenKey

class TokenStore {
	async setToken(key: TokenKey, value: string): Promise<void> {
		await ipc.setToken(key, value)
	}

	async getToken(key: TokenKey): Promise<string | null> {
		return await ipc.getToken(key)
	}

	async getTokenMasked(key: TokenKey): Promise<string | null> {
		return await ipc.getTokenMasked(key)
	}

	async deleteToken(key: TokenKey): Promise<void> {
		await ipc.deleteToken(key)
	}

	async hasToken(key: TokenKey): Promise<boolean> {
		return await ipc.hasToken(key)
	}
}

export const tokenStore = new TokenStore()
