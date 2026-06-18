<script lang="ts">
  import { untrack } from 'svelte';

  import { SourceFetchPhase } from '../gen/shinden_to_anilist/v1/source_pb';
  import type { SourceImportProgress as SourceImportProgressState } from '../domain/anime';

  const recentTitleCount = 5;

  type RecentTitle = {
    id: number;
    title: string;
    tone: string;
  };

  let {
    providerLabel,
    progress,
    onCancel
  }: {
    providerLabel: string;
    progress: SourceImportProgressState | null;
    onCancel: () => void;
  } = $props();

  let now = $state(performance.now());
  let progressPercent = $derived(
    progress === null || progress.total === 0
      ? 0
      : Math.min(100, Math.round((progress.current / progress.total) * 100))
  );
  let elapsedSeconds = $derived(
    progress === null
      ? 0
      : Math.max(0, Math.floor((now - progress.startedAt) / 1000))
  );
  let phaseText = $derived(translatePhase(progress?.phase));
  let progressCountText = $derived(
    progress === null || progress.total === 0
      ? 'Oczekiwanie na dane'
      : `${progress.current} / ${progress.total}`
  );
  let progressTone = $derived(
    `color-mix(in oklab, var(--ctp-mocha-red) ${100 - progressPercent}%, var(--ctp-mocha-green) ${progressPercent}%)`
  );
  let elapsedText = $derived(formatElapsed(elapsedSeconds));
  let recentTitles = $state<RecentTitle[]>([]);
  let recentTitleRows = $derived(recentTitles.slice(-recentTitleCount));

  let lastProgressKey = '';
  let lastRecordedStep = -1;
  let lastRecordedTitle = '';
  let recentTitleId = 0;

  $effect(() => {
    const interval = window.setInterval(() => {
      now = performance.now();
    }, 500);

    return () => window.clearInterval(interval);
  });

  $effect(() => {
    if (progress === null) {
      recentTitles = [];
      lastProgressKey = '';
      lastRecordedStep = -1;
      lastRecordedTitle = '';
      recentTitleId = 0;
      return;
    }

    const progressKey = `${providerLabel}:${progress.startedAt}`;

    if (progressKey !== lastProgressKey) {
      recentTitles = [];
      lastProgressKey = progressKey;
      lastRecordedStep = -1;
      lastRecordedTitle = '';
      recentTitleId = 0;
    }

    const latestTitle = progress.latestTitle.trim();

    if (
      latestTitle === '' ||
      (progress.current === lastRecordedStep &&
        latestTitle === lastRecordedTitle)
    ) {
      return;
    }

    recentTitleId += 1;
    recentTitles = untrack(() =>
      [
        ...recentTitles,
        {
          id: recentTitleId,
          title: latestTitle,
          tone: progressTone
        }
      ].slice(-recentTitleCount)
    );
    lastRecordedStep = progress.current;
    lastRecordedTitle = latestTitle;
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

  function formatElapsed(seconds: number) {
    if (seconds < 60) {
      return `${seconds}s`;
    }

    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;

    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  }
</script>

<div class="source-progress" style:--source-progress-tone={progressTone}>
  <div class="source-progress__header">
    <div class="source-progress__title-group">
      <div class="source-progress__icon" aria-hidden="true">
        <span class="icon-[lucide--download-cloud]"></span>
      </div>
      <div class="source-progress__title-copy">
        <p class="source-progress__eyebrow">Import źródła</p>
        <h2 title={phaseText}>{phaseText}</h2>
        <p
          class="source-progress__provider"
          title={`Importowanie listy z ${providerLabel}`}
        >
          Importowanie listy z {providerLabel}
        </p>
      </div>
    </div>

    <button
      class="btn btn-error btn-soft source-progress__cancel"
      type="button"
      aria-label={`Anuluj import z ${providerLabel}`}
      onclick={onCancel}
    >
      <span aria-hidden="true" class="icon-[lucide--circle-x] size-4"></span>
      <span class="source-progress__cancel-text">Anuluj</span>
    </button>
  </div>

  <div class="source-progress__bar-section">
    <div
      class="source-progress__track"
      role="progressbar"
      aria-label={`Postęp importu ${providerLabel}`}
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={progressPercent}
      style:--source-progress-percent={`${progressPercent}%`}
    >
      <span class="source-progress__fill"></span>
    </div>

    <div class="source-progress__bar-meta">
      <span>{elapsedText}</span>
    </div>
  </div>

  <section
    class="source-progress__recent"
    class:source-progress__recent--waiting={recentTitles.length === 0}
    class:skeleton={recentTitles.length === 0}
    aria-label="Postęp importu"
  >
    {#if recentTitles.length > 0}
      <div class="source-progress__recent-header">
        <span>{progressPercent}%</span>
        <span>{progressCountText}</span>
      </div>
      <div class="source-progress__recent-stream" role="log" aria-live="polite">
        {#each recentTitleRows as row, index (row.id)}
          <div
            class="source-progress__recent-line"
            style:grid-row={recentTitleCount -
              recentTitleRows.length +
              index +
              1}
            style:--source-entry-tone={row.tone}
            title={row.title}
          >
            <span aria-hidden="true" class="source-progress__recent-dot"></span>
            <span class="source-progress__recent-title">{row.title}</span>
          </div>
        {/each}
      </div>
    {:else}
      <div class="source-progress__recent-empty">
        <span class="source-progress__recent-waiting skeleton-text skeleton">
          Wczytywanie danych
        </span>
      </div>
    {/if}
  </section>
</div>

<style>
  .source-progress {
    display: grid;
    width: min(100%, 42rem);
    height: 24rem;
    grid-template-rows: auto auto 1fr;
    gap: calc(var(--spacing) * 5);
    border: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 12%, transparent);
    border-radius: var(--radius-box);
    background: linear-gradient(
      180deg,
      color-mix(in oklab, var(--color-base-100) 96%, white 4%),
      var(--color-base-100)
    );
    padding: calc(var(--spacing) * 6);
    box-shadow:
      0 1.5rem 4rem -2.75rem color-mix(in oklab, black 72%, transparent),
      inset 0 1px 0 color-mix(in oklab, white 10%, transparent);
    overflow: hidden;
  }

  .source-progress__header {
    display: flex;
    min-width: 0;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 4);
    white-space: nowrap;
  }

  .source-progress__icon {
    display: grid;
    width: 3.25rem;
    height: 3.25rem;
    flex: 0 0 auto;
    place-items: center;
    border: var(--border) solid
      color-mix(
        in oklab,
        var(--provider-accent, var(--color-primary)) 38%,
        transparent
      );
    border-radius: var(--radius-box);
    background-color: color-mix(
      in oklab,
      var(--provider-accent, var(--color-primary)) 18%,
      transparent
    );
    color: var(--provider-accent, var(--color-primary));
    font-size: 1.5rem;
  }

  .source-progress__title-group {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: calc(var(--spacing) * 3);
  }

  .source-progress__title-copy {
    display: grid;
    min-width: 0;
    gap: calc(var(--spacing) * 1);
  }

  .source-progress__title-copy > h2,
  .source-progress__eyebrow,
  .source-progress__provider,
  .source-progress__bar-meta,
  .source-progress__recent-header,
  .source-progress__recent-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .source-progress__title-copy > h2 {
    margin: 0;
    font-size: clamp(1.25rem, 1.5vw, 1.65rem);
    font-weight: 800;
    line-height: 1.1;
  }

  .source-progress__eyebrow {
    color: color-mix(in oklab, var(--color-base-content) 62%, transparent);
    font-size: 0.72rem;
    font-weight: 800;
    letter-spacing: 0;
    line-height: 1;
    text-transform: uppercase;
  }

  .source-progress__provider {
    color: color-mix(in oklab, var(--color-base-content) 68%, transparent);
    font-size: 0.9rem;
    font-weight: 650;
    line-height: 1.1;
  }

  .source-progress__cancel {
    flex: 0 0 auto;
  }

  .source-progress__bar-section {
    display: grid;
    min-width: 0;
    gap: calc(var(--spacing) * 3);
  }

  .source-progress__track {
    position: relative;
    height: 1.4rem;
    border: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    border-radius: 999px;
    background-color: var(--color-base-300);
    box-shadow:
      inset 0 0.18rem 0.7rem color-mix(in oklab, black 28%, transparent),
      0 0.65rem 1.4rem -1.15rem
        color-mix(in oklab, var(--source-progress-tone) 86%, transparent);
    overflow: hidden;
  }

  .source-progress__fill {
    position: absolute;
    inset-block: 0;
    inset-inline-start: 0;
    width: var(--source-progress-percent);
    min-width: 0.45rem;
    border-radius: inherit;
    background-color: var(--source-progress-tone);
    box-shadow:
      inset 0 1px 0 color-mix(in oklab, white 26%, transparent),
      0 0 1.1rem -0.4rem
        color-mix(in oklab, var(--source-progress-tone) 80%, transparent);
    transition: width 260ms ease;
  }

  .source-progress__bar-meta {
    display: flex;
    min-width: 0;
    justify-content: flex-end;
    color: color-mix(in oklab, var(--color-base-content) 68%, transparent);
    font-size: 0.86rem;
    font-weight: 750;
    line-height: 1;
    white-space: nowrap;
  }

  .source-progress__bar-meta > span {
    min-width: 4.5ch;
    font-variant-numeric: tabular-nums;
    text-align: end;
  }

  .source-progress__recent {
    display: grid;
    min-height: 0;
    grid-template-rows: auto minmax(0, 1fr);
    gap: calc(var(--spacing) * 3);
    border: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    border-radius: var(--radius-box);
    background-color: color-mix(
      in oklab,
      var(--color-base-200) 74%,
      transparent
    );
    padding: calc(var(--spacing) * 3);
    box-shadow: inset 0 1px 0 color-mix(in oklab, white 8%, transparent);
  }

  .source-progress__recent--waiting {
    position: relative;
    grid-template-rows: minmax(0, 1fr);
    border-color: transparent;
  }

  .source-progress__recent-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 3);
    color: color-mix(in oklab, var(--color-base-content) 76%, transparent);
    font-size: 0.88rem;
    font-weight: 800;
    line-height: 1;
  }

  .source-progress__recent-header > span:first-child {
    color: var(--source-progress-tone);
    font-size: 1rem;
  }

  .source-progress__recent-empty {
    display: grid;
    position: relative;
    z-index: 1;
    min-width: 0;
    min-height: 0;
    place-items: center;
  }

  .source-progress__recent-waiting {
    display: inline-flex;
    max-width: 100%;
    min-height: 2.75rem;
    align-items: center;
    justify-content: center;
    border: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 18%, transparent);
    border-radius: 999px;
    background-color: color-mix(
      in oklab,
      var(--color-base-100) 78%,
      transparent
    );
    color: color-mix(in oklab, var(--color-base-content) 72%, transparent);
    font-size: 1.05rem;
    font-weight: 800;
    line-height: 1;
    overflow: hidden;
    padding-inline: calc(var(--spacing) * 6);
    box-shadow:
      0 0.85rem 1.7rem -1.45rem color-mix(in oklab, black 68%, transparent),
      inset 0 1px 0 color-mix(in oklab, white 8%, transparent);
    text-align: center;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .source-progress__recent-stream {
    display: grid;
    min-height: 0;
    grid-template-rows: repeat(5, minmax(0, 1fr));
    gap: calc(var(--spacing) * 1.5);
    align-content: end;
    overflow: hidden;
  }

  .source-progress__recent-line {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: calc(var(--spacing) * 2);
    border-radius: calc(var(--radius-field) * 0.75);
    background-color: color-mix(
      in oklab,
      var(--source-entry-tone) 12%,
      var(--color-base-200)
    );
    padding-inline: calc(var(--spacing) * 2);
    color: var(--source-entry-tone);
    font-size: 0.9rem;
    font-weight: 650;
    white-space: nowrap;
    overflow: hidden;
  }

  .source-progress__recent-dot {
    width: 0.42rem;
    height: 0.42rem;
    flex: 0 0 auto;
    border-radius: 999px;
    background-color: var(--source-entry-tone);
    box-shadow: 0 0 0.7rem
      color-mix(in oklab, var(--source-entry-tone) 58%, transparent);
  }

  @media (width <= 42rem) {
    .source-progress {
      height: 23rem;
      gap: calc(var(--spacing) * 4);
      padding: calc(var(--spacing) * 4);
    }

    .source-progress__icon {
      width: 2.75rem;
      height: 2.75rem;
      font-size: 1.3rem;
    }

    .source-progress__title-copy > h2 {
      font-size: 1.12rem;
    }

    .source-progress__cancel {
      width: 2.35rem;
      padding-inline: 0;
    }

    .source-progress__cancel-text {
      display: none;
    }

    .source-progress__provider {
      font-size: 0.82rem;
    }
  }
</style>
