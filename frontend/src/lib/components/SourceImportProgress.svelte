<script lang="ts">
  import { SourceFetchPhase } from '../gen/shinden_to_anilist/v1/source_pb';
  import type { SourceImportProgress as SourceImportProgressState } from '../domain/anime';

  let {
    providerLabel,
    progress
  }: {
    providerLabel: string;
    progress: SourceImportProgressState | null;
  } = $props();

  let now = $state(performance.now());
  let progressPercent = $derived(
    progress === null || progress.total === 0
      ? 0
      : Math.min(100, Math.round((progress.current / progress.total) * 100))
  );
  let elapsedSeconds = $derived(
    progress === null ? 0 : Math.max(0, Math.floor((now - progress.startedAt) / 1000))
  );
  let phaseText = $derived(translatePhase(progress?.phase));
  let countText = $derived(
    progress === null || progress.total === 0
      ? 'Przygotowanie'
      : `${progress.current} / ${progress.total}`
  );

  $effect(() => {
    const interval = window.setInterval(() => {
      now = performance.now();
    }, 500);

    return () => window.clearInterval(interval);
  });

  function translatePhase(phase: number | undefined) {
    if (phase === SourceFetchPhase.FETCHING_LIST) {
      return 'Pobieranie listy';
    }

    if (phase === SourceFetchPhase.FETCHING_DETAILS) {
      return 'Pobieranie szczegółów';
    }

    if (phase === SourceFetchPhase.STORING) {
      return 'Zapisywanie listy';
    }

    if (phase === SourceFetchPhase.DONE) {
      return 'Kończenie importu';
    }

    return 'Łączenie ze źródłem';
  }
</script>

<div class="source-progress">
  <div class="source-progress__icon" aria-hidden="true">
    <span class="icon-[lucide--radio-tower]"></span>
  </div>
  <div class="source-progress__body">
    <div class="source-progress__header">
      <p class="text-lg font-bold md:text-2xl">Import z {providerLabel}</p>
      <span class="badge badge-primary">{countText}</span>
    </div>
    <progress
      class="progress progress-primary h-2 w-full"
      value={progressPercent}
      max="100"
      aria-label={`Postęp importu ${providerLabel}`}
    ></progress>
    <div class="source-progress__meta text-sm font-medium text-muted">
      <span>{phaseText}</span>
      <span>{progressPercent}%</span>
      <span>{elapsedSeconds}s</span>
    </div>
    {#if progress?.latestTitle}
      <p class="truncate text-sm text-muted" title={progress.latestTitle}>
        {progress.latestTitle}
      </p>
    {/if}
  </div>
</div>

<style>
  .source-progress {
    display: grid;
    width: min(100%, 34rem);
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    gap: calc(var(--spacing) * 4);
    border: var(--border) solid
      color-mix(in oklab, var(--color-primary) 34%, transparent);
    border-radius: var(--radius-box);
    background-color: color-mix(
      in oklab,
      var(--color-base-200) 88%,
      transparent
    );
    padding: calc(var(--spacing) * 4);
    box-shadow: 0 1rem 2.5rem -2rem
      color-mix(in oklab, var(--color-primary) 60%, transparent);
  }

  .source-progress__icon {
    display: grid;
    width: 3rem;
    height: 3rem;
    place-items: center;
    border-radius: 999px;
    background-color: color-mix(
      in oklab,
      var(--color-primary) 18%,
      transparent
    );
    color: var(--color-primary);
    font-size: 1.5rem;
  }

  .source-progress__body {
    display: grid;
    min-width: 0;
    gap: calc(var(--spacing) * 2);
  }

  .source-progress__header,
  .source-progress__meta {
    display: flex;
    min-width: 0;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 3);
  }

  @media (width <= 36rem) {
    .source-progress {
      grid-template-columns: minmax(0, 1fr);
      justify-items: center;
      text-align: center;
    }

    .source-progress__header,
    .source-progress__meta {
      justify-content: center;
      flex-wrap: wrap;
    }
  }
</style>
