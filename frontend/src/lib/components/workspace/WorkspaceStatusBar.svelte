<script lang="ts">
  import type { MatchListResult } from '../../domain/anime';
  import { createStatusBarBadgeMeasurement } from '../../features/workspace/statusBarBadgeMeasurement.svelte';
  import {
    buildWorkspaceStatusSummary,
    exportButtonText,
    formatDuration
  } from '../../features/workspace/statusBarSummary';
  import type { ExportState } from '../../features/workspace/workspaceController.svelte';

  let {
    entryIds,
    matchResult,
    matchErrorMessage,
    isMatching,
    fetchDurationMs,
    matchDurationMs,
    manualSelections,
    ignoredEntryIds,
    displacedAutomaticEntryIds,
    exportState,
    canExport,
    onExport
  }: {
    entryIds: number[];
    matchResult: MatchListResult | null;
    matchErrorMessage: string | null;
    isMatching: boolean;
    fetchDurationMs: number | null;
    matchDurationMs: number | null;
    manualSelections: Record<number, number>;
    ignoredEntryIds: Record<number, true>;
    displacedAutomaticEntryIds: Record<number, true>;
    exportState: ExportState;
    canExport: boolean;
    onExport: () => void | Promise<void>;
  } = $props();

  let isExportWarningOpen = $state(false);
  let summary = $derived(
    buildWorkspaceStatusSummary(
      entryIds,
      matchResult,
      manualSelections,
      ignoredEntryIds,
      displacedAutomaticEntryIds
    )
  );
  let matchTimeText = $derived(
    isMatching ? '...' : formatDuration(matchDurationMs)
  );
  let exportText = $derived(exportButtonText(exportState));
  const badges = createStatusBarBadgeMeasurement(() => {
    summary.matchedPercentage;
    summary.totalCount;
    summary.reviewCount;
    formatDuration(fetchDurationMs);
    matchTimeText;
  });

  function handleExportClick() {
    if (!canExport) {
      return;
    }

    if (summary.reviewCount > 0) {
      isExportWarningOpen = true;
      return;
    }

    void onExport();
  }

  function closeExportWarning() {
    isExportWarningOpen = false;
  }

  function confirmExportWithUnresolvedEntries() {
    isExportWarningOpen = false;
    void onExport();
  }

  function handleExportWarningKeydown(event: KeyboardEvent) {
    if (!isExportWarningOpen) {
      return;
    }

    if (event.key === 'Escape') {
      event.preventDefault();
      closeExportWarning();
      return;
    }

    if (event.key === 'Enter') {
      event.preventDefault();
      confirmExportWithUnresolvedEntries();
    }
  }
</script>

<footer class="app-status-bar">
  <div class="app-status-bar__body">
    <div
      class="app-status-bar__summary"
      aria-label="Status workspace"
      bind:this={badges.summaryElement}
    >
      {#if matchErrorMessage !== null}
        <span class="text-sm font-medium text-error" data-status-required>
          Błąd dopasowania: {matchErrorMessage}
        </span>
      {/if}
      <span
        class="status-badge status-badge--recognized badge"
        data-status-required
      >
        <span class="status-badge__label">Rozpoznano</span>
        <span class="status-badge__value">{summary.matchedPercentage}%</span>
      </span>
      <span class="status-badge status-badge--total badge" data-status-required>
        <span class="status-badge__label">Wszystkie</span>
        <span class="status-badge__value">{summary.totalCount}</span>
      </span>
      <span
        class="status-badge status-badge--review badge"
        data-status-required
      >
        <span class="status-badge__label">Do sprawdzenia</span>
        <span class="status-badge__value">{summary.reviewCount}</span>
      </span>
      <span
        class:hidden-status-badge={!badges.showApiBadge}
        class:is-measuring={badges.isMeasuringOptionalBadges}
        class="status-badge status-badge--api badge"
        bind:this={badges.apiBadgeElement}
      >
        <span class="status-badge__label">API</span>
        <span class="status-badge__value status-badge__value--duration"
          >{formatDuration(fetchDurationMs)}</span
        >
      </span>
      <span
        class:hidden-status-badge={!badges.showMatchBadge}
        class:is-measuring={badges.isMeasuringOptionalBadges}
        class="status-badge status-badge--match badge"
        bind:this={badges.matchBadgeElement}
      >
        <span class="status-badge__label">Program</span>
        <span class="status-badge__value status-badge__value--duration"
          >{matchTimeText}</span
        >
      </span>
    </div>
    <button
      class="btn btn-error xl:btn-wide"
      class:is-export-ready={summary.reviewCount === 0}
      type="button"
      disabled={!canExport}
      onclick={handleExportClick}
      title={exportState.status === 'error'
        ? exportState.message
        : 'Eksportuj dopasowania do pliku XML'}
    >
      {exportText}
    </button>
  </div>
</footer>

<svelte:window onkeydown={handleExportWarningKeydown} />

<div
  class="modal"
  class:modal-open={isExportWarningOpen}
  role="dialog"
  aria-modal="true"
  aria-labelledby="export-warning-title"
>
  <div class="modal-box max-w-md">
    <div class="flex items-start gap-3">
      <span
        aria-hidden="true"
        class="icon-[lucide--triangle-alert] mt-1 size-5 shrink-0 text-warning"
      ></span>
      <div class="min-w-0">
        <h2 id="export-warning-title" class="text-lg font-bold">
          Nierozwiązane wpisy
        </h2>
        <p class="mt-2 text-sm leading-6 text-muted">
          {summary.reviewCount}
          {summary.reviewCount === 1
            ? 'wpis nie ma dopasowania'
            : 'wpisy nie mają dopasowania'} i nie
          {summary.reviewCount === 1 ? ' zostanie' : ' zostaną'} uwzględnione w eksporcie.
        </p>
      </div>
    </div>
    <div class="modal-action">
      <button class="btn btn-ghost" type="button" onclick={closeExportWarning}>
        Anuluj
      </button>
      <button
        class="btn btn-error"
        type="button"
        onclick={confirmExportWithUnresolvedEntries}
      >
        Eksportuj mimo to
      </button>
    </div>
  </div>
  <button
    class="modal-backdrop"
    type="button"
    aria-label="Zamknij"
    onclick={closeExportWarning}
  ></button>
</div>

<style>
  .app-status-bar {
    flex: 0 0 auto;
    min-width: 0;
    overflow: hidden;
    contain: layout paint;
    border-top: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    background-color: var(--color-base-200);
  }

  .app-status-bar__body {
    display: flex;
    box-sizing: border-box;
    width: 100%;
    min-width: 0;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 4);
    padding-inline: calc(var(--spacing) * 4);
    padding-block: calc(var(--spacing) * 3);
  }

  .app-status-bar__summary {
    position: relative;
    display: flex;
    min-width: 0;
    flex: 1 1 auto;
    flex-wrap: nowrap;
    align-items: center;
    gap: calc(var(--spacing) * 2);
    overflow: hidden;
  }

  .app-status-bar__body > .btn {
    flex: 0 0 auto;
  }

  .app-status-bar__summary > .badge {
    flex: 0 0 auto;
    height: auto;
    min-height: 1.5rem;
    white-space: nowrap;
    text-align: center;
  }

  .app-status-bar__summary > .hidden-status-badge {
    position: absolute;
    visibility: hidden;
    pointer-events: none;
  }

  .app-status-bar__summary > .is-measuring {
    visibility: hidden;
  }

  .is-export-ready {
    position: relative;
    isolation: isolate;
    border-color: transparent;
    background: linear-gradient(
      90deg,
      color-mix(in oklab, #ff6b8a 72%, white 28%) 0rem,
      color-mix(in oklab, #ffd166 76%, white 24%) 8rem,
      color-mix(in oklab, #7bd88f 72%, white 28%) 16rem,
      color-mix(in oklab, #5cc8ff 72%, white 28%) 24rem,
      color-mix(in oklab, #c084fc 72%, white 28%) 32rem,
      color-mix(in oklab, #ff6b8a 72%, white 28%) 40rem
    );
    background-size: 40rem 100%;
    box-shadow:
      0 0 0.5rem color-mix(in oklab, #ffd166 34%, transparent),
      0 0 1.2rem color-mix(in oklab, #5cc8ff 26%, transparent),
      0 0 2rem color-mix(in oklab, #c084fc 20%, transparent);
    color: color-mix(in oklab, var(--color-base-100) 88%, white 12%);
    text-shadow: 0 1px 0 color-mix(in oklab, rgb(2, 2, 2) 22%, transparent);
    animation: export-rainbow-shine 3.2s linear infinite;
  }

  .is-export-ready::after {
    position: absolute;
    inset: -0.25rem;
    z-index: -1;
    border-radius: inherit;
    background:
      radial-gradient(
        circle at 20% 30%,
        color-mix(in oklab, #ffd166 36%, transparent),
        transparent 34%
      ),
      radial-gradient(
        circle at 78% 70%,
        color-mix(in oklab, #5cc8ff 32%, transparent),
        transparent 36%
      ),
      linear-gradient(
        120deg,
        color-mix(in oklab, #ff6b8a 22%, transparent),
        color-mix(in oklab, #7bd88f 20%, transparent),
        color-mix(in oklab, #c084fc 24%, transparent)
      );
    content: '';
    filter: blur(0.65rem);
    opacity: 0.72;
    animation: export-rainbow-glow 2.8s ease-in-out infinite;
  }

  @keyframes export-rainbow-shine {
    0% {
      background-position: 0 50%;
    }

    100% {
      background-position: 40rem 50%;
    }
  }

  @keyframes export-rainbow-glow {
    0%,
    100% {
      opacity: 0.62;
      transform: scale(0.98);
    }

    50% {
      opacity: 0.86;
      transform: scale(1.03);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .is-export-ready,
    .is-export-ready::after {
      animation: none;
    }
  }

  .status-badge {
    --status-time-bridge: color-mix(
      in oklab,
      var(--color-warning) 42%,
      var(--color-error) 58%
    );
    --badge-from: color-mix(in oklab, var(--color-success) 38%, transparent);
    --badge-to: color-mix(in oklab, var(--color-info) 34%, transparent);
    --badge-text-from: color-mix(in oklab, var(--color-success) 66%, white 34%);
    --badge-text-to: color-mix(in oklab, var(--color-info) 66%, white 34%);
    --badge-border: color-mix(
      in oklab,
      var(--badge-to) 70%,
      var(--color-base-content) 16%
    );

    border-color: var(--badge-border);
    background:
      linear-gradient(135deg, var(--badge-from), var(--badge-to)),
      var(--color-base-200);
    gap: calc(var(--spacing) * 1.5);
  }

  .status-badge__label,
  .status-badge__value {
    background: linear-gradient(
      1deg,
      var(--badge-text-from),
      var(--badge-text-to)
    );
    background-clip: text;
    color: transparent;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    filter: saturate(1.5) brightness(1.04) contrast(1.08);
  }

  .status-badge__label {
    font-size: 0.84rem;
    font-weight: 500;
    font-synthesis-weight: auto;
    opacity: 0.8;
  }

  .status-badge__value {
    font-weight: bolder;
    font-synthesis-weight: auto;
    letter-spacing: 0;
  }

  .status-badge__value--duration {
    min-width: 5ch;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .status-badge--recognized {
    --badge-from: color-mix(in oklab, var(--color-success) 42%, transparent);
    --badge-to: color-mix(in oklab, var(--color-info) 28%, transparent);
    --badge-text-from: color-mix(in oklab, var(--color-success) 68%, white 32%);
    --badge-text-to: color-mix(in oklab, var(--color-info) 66%, white 34%);
  }

  .status-badge--total {
    --badge-from: color-mix(in oklab, var(--color-info) 30%, transparent);
    --badge-to: color-mix(
      in oklab,
      var(--provider-accent, var(--color-accent)) 34%,
      transparent
    );
    --badge-text-from: color-mix(in oklab, var(--color-info) 66%, white 34%);
    --badge-text-to: color-mix(
      in oklab,
      var(--provider-accent, var(--color-accent)) 68%,
      white 32%
    );
  }

  .status-badge--review {
    --badge-from: color-mix(
      in oklab,
      var(--provider-accent, var(--color-accent)) 34%,
      transparent
    );
    --badge-to: color-mix(in oklab, var(--color-warning) 34%, transparent);
    --badge-text-from: color-mix(
      in oklab,
      var(--provider-accent, var(--color-accent)) 68%,
      white 32%
    );
    --badge-text-to: color-mix(in oklab, var(--color-warning) 70%, white 30%);
  }

  .status-badge--api {
    --badge-from: color-mix(in oklab, var(--color-warning) 34%, transparent);
    --badge-to: color-mix(in oklab, var(--status-time-bridge) 32%, transparent);
    --badge-text-from: color-mix(in oklab, var(--color-warning) 70%, white 30%);
    --badge-text-to: color-mix(
      in oklab,
      var(--status-time-bridge) 68%,
      white 32%
    );
  }

  .status-badge--match {
    --badge-from: color-mix(
      in oklab,
      var(--status-time-bridge) 32%,
      transparent
    );
    --badge-to: color-mix(in oklab, var(--color-error) 28%, transparent);
    --badge-text-from: color-mix(
      in oklab,
      var(--status-time-bridge) 68%,
      white 32%
    );
    --badge-text-to: color-mix(in oklab, var(--color-error) 66%, white 34%);
  }

  @media (width <= 48rem) {
    .app-status-bar__body {
      align-items: stretch;
      flex-direction: column;
    }

    .app-status-bar__summary {
      justify-content: center;
    }
  }
</style>
