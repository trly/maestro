<script lang="ts">
	import { settingsStore } from "$lib/stores/settingsStore"
	import type * as ipc from "$lib/ipc"
	import { Select } from "bits-ui"
	import { Check, ChevronDown } from "lucide-svelte"

	interface Props {
		availableEditors: ipc.AppInfo[]
		availableTerminals: ipc.TerminalInfo[]
		onStatusChange: (status: { type: "success" | "error"; message: string }) => void
	}

	let { availableEditors, availableTerminals, onStatusChange }: Props = $props()

	let selectedEditorValue = $state<string>("")
	let selectedTerminalValue = $state<string>("")

	$effect(() => {
		const settings = $settingsStore
		selectedEditorValue = settings.selectedEditor || ""
		selectedTerminalValue = settings.selectedTerminal || ""
	})

	let editorNeedsTerminal = $derived(
		availableEditors.find((e) => e.command === selectedEditorValue)?.needsTerminal ?? false
	)

	async function handleEditorChange(value: string | undefined) {
		if (!value) return
		try {
			await settingsStore.setSelectedEditor(value)
			onStatusChange({ type: "success", message: "Editor preference saved" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}

	async function handleTerminalChange(value: string | undefined) {
		if (!value) return
		try {
			await settingsStore.setSelectedTerminal(value)
			onStatusChange({ type: "success", message: "Terminal preference saved" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}
</script>

<div>
	<h3 class="text-lg font-semibold mb-4">Development Tools</h3>
	<p class="text-sm text-muted-foreground mb-6">Configure editor and development tools</p>

	<div class="space-y-6">
		<div>
			<label for="editor-select" class="block text-sm font-medium mb-2">Preferred Editor</label>
			<p class="text-xs text-muted-foreground mb-2">
				Choose your preferred editor for opening worktrees
			</p>
			{#if availableEditors.length > 0}
				<Select.Root
					type="single"
					value={selectedEditorValue}
					onValueChange={handleEditorChange}
					items={availableEditors.map((e) => ({ value: e.command, label: e.displayName }))}
				>
					<Select.Trigger
						id="editor-select"
						class="w-full flex items-center justify-between px-3 py-2 border rounded-md bg-background hover:bg-muted/30 transition-colors"
					>
						<span
							>{availableEditors.find((e) => e.command === selectedEditorValue)?.displayName ||
								"Select an editor"}</span
						>
						<ChevronDown class="w-4 h-4 text-muted-foreground" />
					</Select.Trigger>
					<Select.Content
						class="w-full bg-card border border-border/20 rounded-lg shadow-xl p-1 z-50"
						sideOffset={4}
					>
						{#each availableEditors as editor (editor.command)}
							<Select.Item
								value={editor.command}
								label={editor.displayName}
								class="flex items-center justify-between px-3 py-2 text-sm rounded hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors"
							>
								<span>{editor.displayName}</span>
								{#if selectedEditorValue === editor.command}
									<Check class="w-4 h-4 text-primary" />
								{/if}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			{:else}
				<p class="text-sm text-muted-foreground">No supported editors found in PATH</p>
			{/if}
		</div>

		{#if editorNeedsTerminal}
			<div>
				<label for="terminal-select" class="block text-sm font-medium mb-2"
					>Terminal Application</label
				>
				<p class="text-xs text-muted-foreground mb-2">
					Choose terminal for vim/nvim (macOS only: Terminal or Ghostty)
				</p>
				{#if availableTerminals.length > 0}
					<Select.Root
						type="single"
						value={selectedTerminalValue}
						onValueChange={handleTerminalChange}
						items={availableTerminals.map((t) => ({
							value: t.command,
							label: t.displayName,
						}))}
					>
						<Select.Trigger
							id="terminal-select"
							class="w-full flex items-center justify-between px-3 py-2 border rounded-md bg-background hover:bg-muted/30 transition-colors"
						>
							<span
								>{availableTerminals.find((t) => t.command === selectedTerminalValue)
									?.displayName || "Select a terminal"}</span
							>
							<ChevronDown class="w-4 h-4 text-muted-foreground" />
						</Select.Trigger>
						<Select.Content
							class="w-full bg-card border border-border/20 rounded-lg shadow-xl p-1 z-50"
							sideOffset={4}
						>
							{#each availableTerminals as terminal (terminal.command)}
								<Select.Item
									value={terminal.command}
									label={terminal.displayName}
									class="flex items-center justify-between px-3 py-2 text-sm rounded hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors"
								>
									<span>{terminal.displayName}</span>
									{#if selectedTerminalValue === terminal.command}
										<Check class="w-4 h-4 text-primary" />
									{/if}
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				{:else}
					<p class="text-sm text-muted-foreground">No terminal applications found</p>
				{/if}
			</div>
		{/if}
	</div>
</div>
