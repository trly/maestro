<script lang="ts">
	import { Accordion } from 'bits-ui';

	interface Props {
		repoId: string;
		children: import('svelte').Snippet;
	}

	let { repoId, children }: Props = $props();
</script>

<Accordion.Item value={repoId} class="border border-border rounded-md overflow-hidden mb-4">
	{@render children()}
</Accordion.Item>
