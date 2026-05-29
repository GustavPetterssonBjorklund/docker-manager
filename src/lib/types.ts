export type ContainerState =
	| 'running'
	| 'paused'
	| 'restarting'
	| 'stopped'
	| 'created'
	| 'dead'
	| 'unknown';

export interface ContainerSummary {
	id: string;
	name: string;
	image: string;
	command: string;
	createdAt: string;
	runningFor: string;
	status: string;
	state: ContainerState;
	ports: string;
}

export interface KeyValuePair {
	key: string;
	value: string;
}

export interface ContainerDetails {
	id: string;
	name: string;
	image: string;
	command: string;
	created: string;
	status: string;
	state: ContainerState;
	running: boolean;
	paused: boolean;
	restarting: boolean;
	dead: boolean;
	pid: number;
	exitCode: number;
	startedAt: string | null;
	finishedAt: string | null;
	health: string | null;
	networkMode: string | null;
	ipAddress: string | null;
	labels: KeyValuePair[];
	environment: string[];
	mounts: string[];
	ports: string[];
}

