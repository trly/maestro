<script lang="ts">
	import { AlertDialog } from 'bits-ui';
	import { confirmState, confirmYes, confirmNo } from '$lib/ui/confirm';
</script>

<AlertDialog.Root open={$confirmState.isOpen}>
	<AlertDialog.Portal>
		<AlertDialog.Overlay class="fixed inset-0 z-50 bg-black/40" />
		<AlertDialog.Content class="fixed left-1/2 top-1/2 z-50 -translate-x-1/2 -translate-y-1/2 w-full max-w-lg bg-background rounded-md shadow-2xl border border-border p-6">
			{#if $confirmState.isOpen}
				<AlertDialog.Title class="text-xl font-bold text-foreground mb-3">
					{$confirmState.title}
				</AlertDialog.Title>
				<AlertDialog.Description class="text-muted-foreground mb-6 whitespace-pre-line">
					{$confirmState.message}
				</AlertDialog.Description>
				<div class="flex gap-3 justify-end">
					<AlertDialog.Cancel
						onclick={confirmNo}
						class="px-5 py-2.5 border border-input-border text-foreground rounded-md hover:bg-muted transition-all font-semibold"
					>
						{$confirmState.cancelText || 'Cancel'}
					</AlertDialog.Cancel>
					<AlertDialog.Action
						onclick={confirmYes}
						class="px-5 py-2.5 bg-destructive text-destructive-foreground rounded-md hover:bg-destructive/90 transition-all font-semibold"
					>
						{$confirmState.confirmText || 'Confirm'}
					</AlertDialog.Action>
				</div>
			{/if}
		</AlertDialog.Content>
	</AlertDialog.Portal>
</AlertDialog.Root>
