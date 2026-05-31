<script lang="ts">
	import './layout.css';

	type Provider = 'Shinden' | 'OgladajAnime' | 'AnimeZone';
	const providersValues = new Map<Provider, { name: string; site: string }>([
		[
			'Shinden',
			{
				name: 'Shinden',
				site: 'shinden.pl'
			}
		],
		[
			'OgladajAnime',
			{
				name: 'Oglądaj Anime',
				site: 'ogladajanime.pl'
			}
		],
		[
			'AnimeZone',
			{
				name: 'AnimeZone',
				site: 'animezone.pl'
			}
		]
	]);

	let selectedProvider = $state<Provider>('Shinden');
	let userQuery = $state('');

	function handleSubmit(event: SubmitEvent) {
		event.preventDefault();

		if (!userQuery.trim()) return;

		console.log('Load user list', {
			provider: selectedProvider,
			query: userQuery.trim()
		});
	}
</script>

<main class="flex h-screen flex-col bg-base-300">
	<header class="shrink-0 bg-base-200" style:height="80px">
		<div class="flex grow gap-4 px-4 items-center h-full">
			<div>
				<h1 class="text-xl font-bold">ShindenToAnilist</h1>
				<p class="text-sm font-medium text-base-content/70">Konwerter listy Anime</p>
			</div>
			<div class="flex">
				<div class="join">
					{#each providersValues as [key, provider]}
						<button
							type="button"
							class="btn join-item border-2 border-primary/30 {selectedProvider === key
								? 'btn-primary'
								: 'btn-ghost'}"
							aria-pressed={selectedProvider === key}
							onclick={() => (selectedProvider = key)}
						>
							{provider.name}
						</button>
					{/each}
				</div>
			</div>

			<div class="flex grow">
				<form class="join w-full" onsubmit={handleSubmit}>
					<label class="w-full">
						<input
							class="w-full input join-item focus:ring-0"
							type="text"
							placeholder="ID lub nazwa użytkownika"
							bind:value={userQuery}
							autocomplete="off"
						/>
					</label>
					<button class="btn btn-primary join-item" type="submit" disabled={!userQuery.trim()}>
						Wczytaj
					</button>
				</form>
			</div>
		</div>
	</header>
	<div class="scene p-4">
		<div class="grid-box">
			<div class="grid" aria-hidden="true"></div>
			<div class="grid-layer text-center">
				<p class="font-bold text-4xl">Wczytaj listę, żeby rozpocząć dopasowywanie</p>
				<p class="font-medium text-xl text-base-content/70">
					Aktywny import z Shinden, pozostałe źródła w budowie
				</p>
			</div>
		</div>
	</div>
</main>

<style>
	.scene {
		overflow: hidden;
		width: 100%;
		height: 100%;
	}

	.grid-box {
		position: relative;
		display: grid;
		place-items: center;
		width: 100%;
		height: 100%;
		overflow: hidden;
		border: 2px solid color-mix(in oklab, var(--color-primary) 36%, transparent);
		border-radius: 8px;
		background:
			linear-gradient(
				180deg,
				color-mix(in oklab, var(--color-base-300) 72%, transparent) 0%,
				color-mix(in oklab, var(--color-base-300) 28%, transparent) 42%,
				color-mix(in oklab, var(--color-base-100) 92%, transparent) 100%
			),
			var(--color-base-300);
		box-shadow:
			inset 0 0 34px color-mix(in oklab, var(--color-primary) 18%, transparent),
			0 18px 44px rgb(0 0 0 / 0.22);
		transform-style: preserve-3d;
	}

	.grid-box::before,
	.grid-box::after {
		position: absolute;
		z-index: 1;
		inset: 0;
		content: '';
		pointer-events: none;
	}

	.grid-box::before {
		background: linear-gradient(
			180deg,
			var(--color-base-300) 0%,
			color-mix(in oklab, var(--color-base-300) 72%, transparent) 20%,
			transparent 52%
		);
	}

	.grid-box::after {
		background: radial-gradient(
			ellipse at center,
			transparent 42%,
			color-mix(in oklab, var(--color-base-300) 82%, transparent) 100%
		);
	}

	.grid {
		position: absolute;
		z-index: 0;
		inset: 0;
		width: 100%;
		height: 100%;
		background-image:
			linear-gradient(
				color-mix(in oklab, var(--color-primary) 20%, transparent) 2px,
				transparent 3px
			),
			linear-gradient(
				90deg,
				color-mix(in oklab, var(--color-primary) 20%, transparent) 2px,
				transparent 3px
			);
		background-size: 64px 64px;

		animation: moveGrid 7.2s linear infinite;
	}

	.grid-layer {
		position: relative;
		z-index: 2;
		display: grid;
		gap: 0.4rem;
		justify-items: center;
	}

	@keyframes moveGrid {
		from {
			background-position: 0 0;
		}
		to {
			background-position: 64px 256px;
		}
	}
</style>
