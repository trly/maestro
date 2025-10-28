import * as ipc from "./ipc"

export type TokenKey = ipc.TokenKey
export type AllTokens = ipc.AllTokens

class TokenStore {
	// Cache tokens to avoid multiple keychain prompts
	private tokenCache: AllTokens | null = null

	async setToken(key: TokenKey, value: string): Promise<void> {
		await ipc.setToken(key, value)
		// Invalidate cache
		this.tokenCache = null
	}

	async deleteToken(key: TokenKey): Promise<void> {
		await ipc.deleteToken(key)
		// Invalidate cache
		this.tokenCache = null
	}

	/**
	 * Load all tokens in a single keychain access (prevents multiple prompts)
	 */
	async getAllTokens(): Promise<AllTokens> {
		if (!this.tokenCache) {
			this.tokenCache = await ipc.getAllTokens()
		}
		return this.tokenCache
	}

	/**
	 * Load all tokens masked in a single keychain access
	 */
	async getAllTokensMasked(): Promise<AllTokens> {
		return await ipc.getAllTokensMasked()
	}

	/**
	 * Clear the token cache (useful after updates)
	 */
	clearCache(): void {
		this.tokenCache = null
	}
}

export const tokenStore = new TokenStore()
