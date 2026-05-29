<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke, isTauri } from '@tauri-apps/api/core';
	import { formatAge, formatError, shortId, stateLabel, stateTone } from '$lib/format';
	import type { ContainerDetails, ContainerState, ContainerSummary } from '$lib/types';

	const AUTO_REFRESH_MS = 5_000;

	let containers: ContainerSummary[] = [];
	let filter = '';
	let stateFilter: ContainerState | 'all' = 'all';
	let listLoading = false;
	let actionBusyId: string | null = null;
	let listError = '';
	let autoRefreshEnabled = true;
	let inTauri = isTauri();
	let filteredContainers: ContainerSummary[] = [];
	let runningCount = 0;
	let stoppedCount = 0;
	let pausedCount = 0;
	let totalCount = 0;

	$: filteredContainers = containers.filter((container) => {
		const matchesState = stateFilter === 'all' || container.state === stateFilter;
		const query = filter.trim().toLowerCase();
		const matchesQuery =
			!query ||
			container.name.toLowerCase().includes(query) ||
			container.image.toLowerCase().includes(query) ||
			container.id.toLowerCase().includes(query) ||
			container.status.toLowerCase().includes(query);

		return matchesState && matchesQuery;
	});

	$: runningCount = containers.filter((container) => container.state === 'running').length;
	$: stoppedCount = containers.filter((container) => container.state === 'stopped').length;
	$: pausedCount = containers.filter((container) => container.state === 'paused').length;
	$: totalCount = containers.length;

	function canStart(container: ContainerSummary) {
		return container.state !== 'running';
	}

	function displayError(error: unknown) {
		return formatError(error);
	}

	async function loadContainers(options: { quiet?: boolean } = {}) {
		if (!inTauri) {
			return;
		}

		listLoading = !options.quiet;
		listError = '';

		try {
			const next = await invoke<ContainerSummary[]>('list_containers', { all: true });
			containers = next;
		} catch (error) {
			listError = displayError(error);
		} finally {
			listLoading = false;
		}
	}

	async function runAction(action: 'start_container' | 'stop_container', id: string) {
		if (!inTauri) {
			return;
		}

		actionBusyId = id;
		listError = '';

		try {
			await invoke<ContainerDetails>(action, { id });
			await loadContainers({ quiet: true });
		} catch (error) {
			listError = displayError(error);
		} finally {
			actionBusyId = null;
		}
	}

	async function refreshAll() {
		await loadContainers();
	}

	onMount(() => {
		void refreshAll();

		if (!autoRefreshEnabled) {
			return;
		}

		const interval = window.setInterval(() => {
			void loadContainers({ quiet: true });
		}, AUTO_REFRESH_MS);

		return () => window.clearInterval(interval);
	});
</script>

<svelte:head>
	<title>Docker Manager</title>
	<meta
		name="description"
		content="A simple Tauri + Svelte utility for inspecting and controlling local Docker containers."
	/>
</svelte:head>

<main class="shell">
	<section class="hero">
		<div>
			<p class="eyebrow">Local Docker control</p>
			<h1>Docker Manager</h1>
			<p class="lede">
				Keep your services in a simple card grid, with the current state and start/stop actions
				always visible.
			</p>
		</div>

		<div class="hero-meta">
			<div class="pill-group">
				<span class="pill">Tauri {inTauri ? 'connected' : 'preview only'}</span>
				<span class="pill">{totalCount} containers</span>
				<span class="pill">Auto refresh {autoRefreshEnabled ? 'on' : 'off'}</span>
			</div>
			<button class="button button-secondary" on:click={refreshAll} disabled={listLoading}>
				{listLoading ? 'Refreshing…' : 'Refresh'}
			</button>
		</div>
	</section>

	{#if !inTauri}
		<section class="notice">
			<strong>Open this app inside Tauri.</strong>
			<span>The web preview is only for layout work; Docker commands are available in the desktop shell.</span>
		</section>
	{/if}

	<section class="stats">
		<article class="stat-card">
			<span>Running</span>
			<strong>{runningCount}</strong>
		</article>
		<article class="stat-card">
			<span>Stopped</span>
			<strong>{stoppedCount}</strong>
		</article>
		<article class="stat-card">
			<span>Paused</span>
			<strong>{pausedCount}</strong>
		</article>
		<article class="stat-card">
			<span>Total</span>
			<strong>{totalCount}</strong>
		</article>
	</section>

	{#if listError}
		<section class="banner error">
			<span>Docker list failed</span>
			<p>{listError}</p>
		</section>
	{/if}

	<section class="toolbar">
		<label class="field search">
			<span>Search</span>
			<input bind:value={filter} type="search" placeholder="Name, image, id, status" />
		</label>

		<label class="field">
			<span>Status</span>
			<select bind:value={stateFilter}>
				<option value="all">All states</option>
				<option value="running">Running</option>
				<option value="paused">Paused</option>
				<option value="restarting">Restarting</option>
				<option value="stopped">Stopped</option>
				<option value="created">Created</option>
				<option value="dead">Dead</option>
				<option value="unknown">Unknown</option>
			</select>
		</label>
	</section>

	<section class="services-panel panel">
		<header class="panel-header">
			<div>
				<h2>Services</h2>
				<p>{filteredContainers.length} visible</p>
			</div>
			<div class="panel-actions">
				<span class="subtle">{listLoading ? 'Syncing' : 'Idle'}</span>
			</div>
		</header>

		{#if filteredContainers.length === 0}
			<div class="empty-state">
				<p>No containers match the current filter.</p>
			</div>
		{:else}
			<div class="cards">
				{#each filteredContainers as container}
					<article class="service-card">
						<div class="card-top">
							<div class="card-title">
								<strong>{container.name}</strong>
								<span class={`status-pill tone-${stateTone(container.state)}`}>
									{stateLabel(container.state)}
								</span>
							</div>
							<p class="muted mono">{container.image}</p>
						</div>

						<div class="card-meta">
							<div>
								<span>Container</span>
								<strong class="mono">{shortId(container.id)}</strong>
							</div>
							<div>
								<span>Uptime</span>
								<strong>{formatAge(container.runningFor)}</strong>
							</div>
							<div>
								<span>Ports</span>
								<strong class="mono">{container.ports || 'none'}</strong>
							</div>
							<div>
								<span>Status</span>
								<strong>{container.status}</strong>
							</div>
						</div>

						<div class="card-actions">
							{#if canStart(container)}
								<button
									class="button button-primary"
									disabled={actionBusyId === container.id}
									on:click={() => runAction('start_container', container.id)}
								>
									{actionBusyId === container.id ? 'Starting…' : 'Start'}
								</button>
							{:else}
								<button
									class="button button-secondary"
									disabled={actionBusyId === container.id}
									on:click={() => runAction('stop_container', container.id)}
								>
									{actionBusyId === container.id ? 'Stopping…' : 'Stop'}
								</button>
							{/if}
						</div>
					</article>
				{/each}
			</div>
		{/if}
	</section>
</main>
