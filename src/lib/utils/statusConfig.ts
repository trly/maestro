import type { ComponentType } from 'svelte'
import {
	CheckCircle2,
	XCircle,
	Clock,
	Loader2,
	Ban,
	AlertCircle,
	GitCommit,
	GitBranch
} from 'lucide-svelte'
import type { ExecutionStatus, ValidationStatus, CommitStatus, AnalysisStatus } from '$lib/types'

export interface StatusConfig {
	Icon: ComponentType
	class: string
	bgClass?: string
	textClass?: string
	label?: string
}

export function getExecutionStatusConfig(status: ExecutionStatus): StatusConfig {
	switch (status) {
		case 'running':
			return { Icon: Loader2, class: 'text-primary animate-spin' }
		case 'completed':
			return { Icon: CheckCircle2, class: 'text-success' }
		case 'failed':
			return { Icon: XCircle, class: 'text-destructive' }
		case 'cancelled':
			return { Icon: Ban, class: 'text-warning' }
		case 'pending':
			return { Icon: Clock, class: 'text-muted-foreground' }
		default:
			return { Icon: Clock, class: 'text-muted-foreground' }
	}
}

export function getValidationStatusConfig(status: ValidationStatus | null): StatusConfig | null {
	if (!status) return null
	
	switch (status) {
		case 'running':
			return { Icon: Loader2, class: 'text-primary animate-spin' }
		case 'passed':
			return { Icon: CheckCircle2, class: 'text-success' }
		case 'failed':
			return { Icon: XCircle, class: 'text-destructive' }
		case 'pending':
			return { Icon: Clock, class: 'text-muted-foreground' }
		case 'cancelled':
			return { Icon: Ban, class: 'text-warning' }
		default:
			return null
	}
}

export function getCommitStatusConfig(status: CommitStatus): StatusConfig | null {
	switch (status) {
		case 'committed':
			return { Icon: GitCommit, class: 'text-success' }
		case 'uncommitted':
			return { Icon: GitBranch, class: 'text-warning' }
		case 'none':
			return null
		default:
			return null
	}
}

export function getAnalysisStatusConfig(status: AnalysisStatus): StatusConfig {
	switch (status) {
		case 'completed':
			return {
				Icon: CheckCircle2,
				class: 'text-success',
				bgClass: 'bg-success/10',
				textClass: 'text-success',
				label: 'Completed'
			}
		case 'failed':
			return {
				Icon: AlertCircle,
				class: 'text-destructive',
				bgClass: 'bg-destructive/10',
				textClass: 'text-destructive',
				label: 'Failed'
			}
		case 'running':
			return {
				Icon: Loader2,
				class: 'text-primary animate-spin',
				bgClass: 'bg-primary/10',
				textClass: 'text-primary',
				label: 'Running'
			}
		case 'pending':
		default:
			return {
				Icon: Clock,
				class: 'text-muted-foreground',
				bgClass: 'bg-muted',
				textClass: 'text-muted-foreground',
				label: 'Pending'
			}
	}
}
