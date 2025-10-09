<script lang="ts">
	import { diffLines, type Change } from 'diff';

	interface Props {
		oldText: string;
		newText: string;
		onupdate: (text: string) => void;
	}

	let { oldText, newText, onupdate }: Props = $props();

	let diff = $derived(diffLines(oldText, newText));

	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		onupdate(target.value);
	}
</script>

<div class="grid grid-cols-2 gap-4">
	<div class="space-y-2">
		<h3 class="text-sm font-semibold text-gray-600">Previous Revision</h3>
		<div class="bg-gray-50 border-2 border-gray-200 rounded-xl p-4 font-mono text-sm overflow-auto max-h-96">
			{#each diff as change}
				{#if change.removed}
					<div class="bg-red-100 text-red-800 -mx-4 px-4">{change.value}</div>
				{:else if !change.added}
					<div class="text-gray-700">{change.value}</div>
				{/if}
			{/each}
		</div>
	</div>

	<div class="space-y-2">
		<h3 class="text-sm font-semibold text-gray-600">New Revision</h3>
		<textarea
			value={newText}
			oninput={handleInput}
			placeholder="Edit your prompt..."
			rows="12"
			class="w-full px-4 py-3 border-2 border-indigo-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all resize-y font-mono text-sm"
		></textarea>
	</div>
</div>

<div class="mt-4 p-4 bg-blue-50 border-2 border-blue-200 rounded-xl">
	<h3 class="text-sm font-semibold text-gray-700 mb-2">Unified Diff</h3>
	<div class="font-mono text-xs overflow-auto max-h-48">
		{#each diff as change}
			{#if change.added}
				<div class="bg-green-100 text-green-800">+ {change.value}</div>
			{:else if change.removed}
				<div class="bg-red-100 text-red-800">- {change.value}</div>
			{:else}
				<div class="text-gray-600">  {change.value}</div>
			{/if}
		{/each}
	</div>
</div>
