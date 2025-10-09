
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * Environment variables [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env`. Like [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), this module cannot be imported into client-side code. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * _Unlike_ [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), the values exported from this module are statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * ```ts
 * import { API_KEY } from '$env/static/private';
 * ```
 * 
 * Note that all environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * 
 * ```
 * MY_FEATURE_FLAG=""
 * ```
 * 
 * You can override `.env` values from the command line like so:
 * 
 * ```sh
 * MY_FEATURE_FLAG="enabled" npm run dev
 * ```
 */
declare module '$env/static/private' {
	export const VITE_MAESTRO_GITHUB_TOKEN: string;
	export const STARSHIP_SHELL: string;
	export const MANPATH: string;
	export const __MISE_DIFF: string;
	export const GHOSTTY_RESOURCES_DIR: string;
	export const TERM_PROGRAM: string;
	export const NODE: string;
	export const SHELL: string;
	export const TERM: string;
	export const TMPDIR: string;
	export const SRC_ENDPOINT: string;
	export const TERM_PROGRAM_VERSION: string;
	export const npm_config_local_prefix: string;
	export const ZSH: string;
	export const USER: string;
	export const LS_COLORS: string;
	export const COMMAND_MODE: string;
	export const SSH_AUTH_SOCK: string;
	export const __CF_USER_TEXT_ENCODING: string;
	export const npm_execpath: string;
	export const SRC_BATCH_TMP_DIR: string;
	export const PAGER: string;
	export const LSCOLORS: string;
	export const AMP_CLIENT_ID: string;
	export const PATH: string;
	export const npm_package_json: string;
	export const _: string;
	export const GHOSTTY_SHELL_FEATURES: string;
	export const __CFBundleIdentifier: string;
	export const npm_command: string;
	export const PWD: string;
	export const DISABLE_AUTO_UPDATE: string;
	export const npm_lifecycle_event: string;
	export const npm_package_name: string;
	export const LANG: string;
	export const SRC_ACCESS_TOKEN: string;
	export const AMP_CLIENT_SECRET: string;
	export const XPC_FLAGS: string;
	export const npm_package_version: string;
	export const AMP_API_KEY: string;
	export const XPC_SERVICE_NAME: string;
	export const HOME: string;
	export const SHLVL: string;
	export const __MISE_ORIG_PATH: string;
	export const TERMINFO: string;
	export const ATUIN_HISTORY_ID: string;
	export const HOMEBREW_PREFIX: string;
	export const MISE_SHELL: string;
	export const STARSHIP_SESSION_KEY: string;
	export const LESS: string;
	export const LOGNAME: string;
	export const npm_lifecycle_script: string;
	export const ATUIN_SESSION: string;
	export const XDG_DATA_DIRS: string;
	export const FZF_DEFAULT_COMMAND: string;
	export const GHOSTTY_BIN_DIR: string;
	export const npm_config_user_agent: string;
	export const __MISE_SESSION: string;
	export const OSLogRateLimit: string;
	export const npm_node_execpath: string;
	export const COLORTERM: string;
	export const NODE_ENV: string;
}

/**
 * Similar to [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private), except that it only includes environment variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Values are replaced statically at build time.
 * 
 * ```ts
 * import { PUBLIC_BASE_URL } from '$env/static/public';
 * ```
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to runtime environment variables, as defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * This module cannot be imported into client-side code.
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * console.log(env.DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` always includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 */
declare module '$env/dynamic/private' {
	export const env: {
		VITE_MAESTRO_GITHUB_TOKEN: string;
		STARSHIP_SHELL: string;
		MANPATH: string;
		__MISE_DIFF: string;
		GHOSTTY_RESOURCES_DIR: string;
		TERM_PROGRAM: string;
		NODE: string;
		SHELL: string;
		TERM: string;
		TMPDIR: string;
		SRC_ENDPOINT: string;
		TERM_PROGRAM_VERSION: string;
		npm_config_local_prefix: string;
		ZSH: string;
		USER: string;
		LS_COLORS: string;
		COMMAND_MODE: string;
		SSH_AUTH_SOCK: string;
		__CF_USER_TEXT_ENCODING: string;
		npm_execpath: string;
		SRC_BATCH_TMP_DIR: string;
		PAGER: string;
		LSCOLORS: string;
		AMP_CLIENT_ID: string;
		PATH: string;
		npm_package_json: string;
		_: string;
		GHOSTTY_SHELL_FEATURES: string;
		__CFBundleIdentifier: string;
		npm_command: string;
		PWD: string;
		DISABLE_AUTO_UPDATE: string;
		npm_lifecycle_event: string;
		npm_package_name: string;
		LANG: string;
		SRC_ACCESS_TOKEN: string;
		AMP_CLIENT_SECRET: string;
		XPC_FLAGS: string;
		npm_package_version: string;
		AMP_API_KEY: string;
		XPC_SERVICE_NAME: string;
		HOME: string;
		SHLVL: string;
		__MISE_ORIG_PATH: string;
		TERMINFO: string;
		ATUIN_HISTORY_ID: string;
		HOMEBREW_PREFIX: string;
		MISE_SHELL: string;
		STARSHIP_SESSION_KEY: string;
		LESS: string;
		LOGNAME: string;
		npm_lifecycle_script: string;
		ATUIN_SESSION: string;
		XDG_DATA_DIRS: string;
		FZF_DEFAULT_COMMAND: string;
		GHOSTTY_BIN_DIR: string;
		npm_config_user_agent: string;
		__MISE_SESSION: string;
		OSLogRateLimit: string;
		npm_node_execpath: string;
		COLORTERM: string;
		NODE_ENV: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * Similar to [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), but only includes variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Note that public dynamic environment variables must all be sent from the server to the client, causing larger network requests — when possible, use `$env/static/public` instead.
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.PUBLIC_DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
