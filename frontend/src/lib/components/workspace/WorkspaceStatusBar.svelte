<script lang="ts">
  import type { MatchListResult, ShindenEntry } from "../../domain/anime";

  let {
    entries,
    matchResult,
    matchErrorMessage,
    isMatching,
    fetchDurationMs,
    matchDurationMs,
    manualSelections = $bindable(),
  }: {
    entries: ShindenEntry[];
    matchResult: MatchListResult | null;
    matchErrorMessage: string | null;
    isMatching: boolean;
    fetchDurationMs: number | null;
    matchDurationMs: number | null;
    manualSelections: Record<number, number>;
  } = $props();

  let totalCount = $derived(entries.length);
  let automaticWinnerIds = $derived.by(() => {
    const ids = new Set<number>();

    for (const entry of matchResult?.entries ?? []) {
      if (entry.result.winner !== null) {
        ids.add(entry.shindenId);
      }
    }

    return ids;
  });
  let automaticallyMatchedCount = $derived(automaticWinnerIds.size);
  let manuallySelectedCount = $derived.by(() => {
    let count = 0;

    for (const entry of entries) {
      if (
        !automaticWinnerIds.has(entry.id) &&
        manualSelections[entry.id] !== undefined
      ) {
        count += 1;
      }
    }

    return count;
  });
  let reviewCount = $derived(
    Math.max(0, totalCount - automaticallyMatchedCount - manuallySelectedCount),
  );
  let matchedPercentage = $derived(
    totalCount > 0
      ? Math.round((automaticallyMatchedCount / totalCount) * 100)
      : 0,
  );
  let matchTimeText = $derived(
    isMatching ? "..." : formatDuration(matchDurationMs),
  );

  function formatDuration(durationMs: number | null) {
    if (durationMs === null) return "-";

    if (durationMs < 1000) {
      return `${Math.round(durationMs)} ms`;
    }

    return `${(durationMs / 1000).toFixed(1)} s`;
  }
</script>

<footer class="app-status-bar">
  <div class="app-status-bar__body">
    <div class="app-status-bar__summary" aria-label="Status workspace">
      {#if matchErrorMessage !== null}
        <span class="text-sm font-medium text-error">
          Błąd dopasowania: {matchErrorMessage}
        </span>
      {/if}
      <span class="badge status-badge status-badge--recognized">
        <span class="status-badge__text">Rozpoznano: {matchedPercentage}%</span>
      </span>
      <span class="badge status-badge status-badge--total">
        <span class="status-badge__text">Wszystkie: {totalCount}</span>
      </span>
      <span class="badge status-badge status-badge--review">
        <span class="status-badge__text">Do sprawdzenia: {reviewCount}</span>
      </span>
      <span class="badge status-badge status-badge--api">
        <span class="status-badge__text">
          API: {formatDuration(fetchDurationMs)}
        </span>
      </span>
      <span class="badge status-badge status-badge--match">
        <span class="status-badge__text">Dopasowano w: {matchTimeText}</span>
      </span>
    </div>
    <button class="btn btn-error md:btn-wide" type="button">Eksport</button>
  </div>
</footer>

<style>
  .app-status-bar {
    border-top: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    background-color: var(--color-base-200);
  }

  .app-status-bar__body {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 4);
    padding-inline: calc(var(--spacing) * 4);
    padding-block: calc(var(--spacing) * 3);
  }

  .app-status-bar__summary {
    display: flex;
    min-width: 0;
    flex-wrap: wrap;
    align-items: center;
    gap: calc(var(--spacing) * 2);
  }

  .app-status-bar__summary > .badge {
    height: auto;
    min-height: 1.5rem;
    white-space: normal;
    text-align: center;
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
  }

  .status-badge__text {
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
  }
</style>
