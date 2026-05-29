import type { ContainerState } from './types';

export function formatError(error: unknown): string {
	if (error instanceof Error) {
		return error.message;
	}

	if (typeof error === 'string') {
		return error;
	}

	try {
		return JSON.stringify(error);
	} catch {
		return 'Unknown error';
	}
}

export function shortId(id: string): string {
	return id.slice(0, 12);
}

export function stateLabel(state: ContainerState): string {
	switch (state) {
		case 'running':
			return 'Running';
		case 'paused':
			return 'Paused';
		case 'restarting':
			return 'Restarting';
		case 'stopped':
			return 'Stopped';
		case 'created':
			return 'Created';
		case 'dead':
			return 'Dead';
		default:
			return 'Unknown';
	}
}

export function stateTone(state: ContainerState): 'success' | 'warning' | 'danger' | 'neutral' {
	switch (state) {
		case 'running':
			return 'success';
		case 'paused':
		case 'created':
			return 'warning';
		case 'restarting':
		case 'dead':
			return 'danger';
		default:
			return 'neutral';
	}
}

export function formatDateTime(value: string | null | undefined): string {
	if (!value) {
		return 'n/a';
	}

	const date = new Date(value);

	if (Number.isNaN(date.getTime())) {
		return value;
	}

	return new Intl.DateTimeFormat(undefined, {
		dateStyle: 'medium',
		timeStyle: 'short'
	}).format(date);
}

export function formatAge(value: string | null | undefined): string {
	if (!value) {
		return 'n/a';
	}

	const normalized = value.trim();

	if (!normalized) {
		return 'n/a';
	}

	return normalized;
}

