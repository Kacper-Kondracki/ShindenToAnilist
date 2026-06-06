<script lang="ts">
  import { onMount } from "svelte";

  import type { MatchListResult } from "../../domain/anime";

  let {
    entryIds,
    matchResult,
    matchErrorMessage,
    isMatching,
    fetchDurationMs,
    matchDurationMs,
    manualSelections = $bindable(),
  }: {
    entryIds: number[];
    matchResult: MatchListResult | null;
    matchErrorMessage: string | null;
    isMatching: boolean;
    fetchDurationMs: number | null;
    matchDurationMs: number | null;
    manualSelections: Record<number, number>;
  } = $props();

  let totalCount = $derived(entryIds.length);
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

    for (const entryId of entryIds) {
      if (
        !automaticWinnerIds.has(entryId) &&
        manualSelections[entryId] !== undefined
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
  let summaryElement: HTMLDivElement;
  let apiBadgeElement: HTMLSpanElement;
  let matchBadgeElement: HTMLSpanElement;
  let showApiBadge = $state(true);
  let showMatchBadge = $state(true);
  let isMeasuringOptionalBadges = $state(true);
  let apiBadgeWidth = 0;
  let matchBadgeWidth = 0;
  let measureAnimationFrame = 0;

  onMount(() => {
    const resizeObserver = new ResizeObserver(scheduleOptionalBadgeMeasurement);
    resizeObserver.observe(summaryElement);
    scheduleOptionalBadgeMeasurement();
    return () => {
      resizeObserver.disconnect();
      cancelAnimationFrame(measureAnimationFrame);
    };
  });

  $effect(() => {
    matchedPercentage;
    totalCount;
    reviewCount;
    formatDuration(fetchDurationMs);
    matchTimeText;

    scheduleOptionalBadgeMeasurement();
  });

  function scheduleOptionalBadgeMeasurement() {
    cancelAnimationFrame(measureAnimationFrame);
    measureAnimationFrame = requestAnimationFrame(measureOptionalBadges);
  }

  function measureOptionalBadges() {
    if (!summaryElement || !apiBadgeElement || !matchBadgeElement) return;

    if (apiBadgeElement.offsetWidth > 0) {
      apiBadgeWidth = apiBadgeElement.getBoundingClientRect().width;
    }

    if (matchBadgeElement.offsetWidth > 0) {
      matchBadgeWidth = matchBadgeElement.getBoundingClientRect().width;
    }

    const requiredBadges = Array.from(
      summaryElement.querySelectorAll<HTMLElement>("[data-status-required]"),
    );
    const visibleRequiredBadges = requiredBadges.filter(
      (badge) => badge.offsetWidth > 0,
    );
    const gap = Number.parseFloat(getComputedStyle(summaryElement).columnGap);
    const badgeGap = Number.isFinite(gap) ? gap : 0;
    const requiredWidth = visibleRequiredBadges.reduce(
      (width, badge) => width + badge.getBoundingClientRect().width,
      0,
    );
    const availableWidth = summaryElement.getBoundingClientRect().width;

    let visibleBadgeCount = visibleRequiredBadges.length;
    let nextWidth = requiredWidth;

    const apiFits = fitsBadge(nextWidth, visibleBadgeCount, apiBadgeWidth);

    if (apiFits) {
      nextWidth += gapBeforeNextBadge(visibleBadgeCount) + apiBadgeWidth;
      visibleBadgeCount += 1;
    }

    const matchFits =
      apiFits && fitsBadge(nextWidth, visibleBadgeCount, matchBadgeWidth);

    showApiBadge = apiFits;
    showMatchBadge = matchFits;
    isMeasuringOptionalBadges = false;

    function fitsBadge(
      currentWidth: number,
      currentBadgeCount: number,
      badgeWidth: number,
    ) {
      if (badgeWidth <= 0) return false;

      return (
        currentWidth + gapBeforeNextBadge(currentBadgeCount) + badgeWidth <=
        availableWidth
      );
    }

    function gapBeforeNextBadge(currentBadgeCount: number) {
      return currentBadgeCount > 0 ? badgeGap : 0;
    }
  }

  function formatDuration(durationMs: number | null) {
    if (durationMs === null) return "--.--s";

    return `${(durationMs / 1000).toFixed(2)}s`;
  }
</script>

<footer class="app-status-bar">
  <div class="app-status-bar__body">
    <div
      class="app-status-bar__summary"
      aria-label="Status workspace"
      bind:this={summaryElement}
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
        <span class="status-badge__value">{matchedPercentage}%</span>
      </span>
      <span class="status-badge status-badge--total badge" data-status-required>
        <span class="status-badge__label">Wszystkie</span>
        <span class="status-badge__value">{totalCount}</span>
      </span>
      <span
        class="status-badge status-badge--review badge"
        data-status-required
      >
        <span class="status-badge__label">Do sprawdzenia</span>
        <span class="status-badge__value">{reviewCount}</span>
      </span>
      <span
        class:hidden-status-badge={!showApiBadge}
        class:is-measuring={isMeasuringOptionalBadges}
        class="status-badge status-badge--api badge"
        bind:this={apiBadgeElement}
      >
        <span class="status-badge__label">API</span>
        <span class="status-badge__value status-badge__value--duration"
          >{formatDuration(fetchDurationMs)}</span
        >
      </span>
      <span
        class:hidden-status-badge={!showMatchBadge}
        class:is-measuring={isMeasuringOptionalBadges}
        class="status-badge status-badge--match badge"
        bind:this={matchBadgeElement}
      >
        <span class="status-badge__label">Program</span>
        <span class="status-badge__value status-badge__value--duration"
          >{matchTimeText}</span
        >
      </span>
    </div>
    <button class="btn btn-error xl:btn-wide" type="button">Eksport</button>
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
    position: relative;
    display: flex;
    min-width: 0;
    flex: 1 1 auto;
    flex-wrap: nowrap;
    align-items: center;
    gap: calc(var(--spacing) * 2);
  }

  .app-status-bar__summary > .badge {
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
