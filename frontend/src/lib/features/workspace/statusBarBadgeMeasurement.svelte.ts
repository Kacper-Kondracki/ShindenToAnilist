import { onMount } from 'svelte';

export type StatusBarBadgeMeasurement = ReturnType<
  typeof createStatusBarBadgeMeasurement
>;

export function createStatusBarBadgeMeasurement(
  onMeasurementInput: () => void
) {
  let summaryElement = $state<HTMLDivElement | null>(null);
  let apiBadgeElement = $state<HTMLSpanElement | null>(null);
  let matchBadgeElement = $state<HTMLSpanElement | null>(null);
  let showApiBadge = $state(true);
  let showMatchBadge = $state(true);
  let isMeasuringOptionalBadges = $state(true);
  let apiBadgeWidth = 0;
  let matchBadgeWidth = 0;
  let measureAnimationFrame = 0;

  onMount(() => {
    if (summaryElement === null) {
      return;
    }

    const resizeObserver = new ResizeObserver(schedule);
    resizeObserver.observe(summaryElement);
    schedule();

    return () => {
      resizeObserver.disconnect();
      cancelAnimationFrame(measureAnimationFrame);
    };
  });

  $effect(() => {
    onMeasurementInput();
    schedule();
  });

  function schedule() {
    cancelAnimationFrame(measureAnimationFrame);
    measureAnimationFrame = requestAnimationFrame(measure);
  }

  function measure() {
    if (
      summaryElement === null ||
      apiBadgeElement === null ||
      matchBadgeElement === null
    ) {
      return;
    }

    if (apiBadgeElement.offsetWidth > 0) {
      apiBadgeWidth = apiBadgeElement.getBoundingClientRect().width;
    }

    if (matchBadgeElement.offsetWidth > 0) {
      matchBadgeWidth = matchBadgeElement.getBoundingClientRect().width;
    }

    const requiredBadges = Array.from(
      summaryElement.querySelectorAll<HTMLElement>('[data-status-required]')
    );
    const visibleRequiredBadges = requiredBadges.filter(
      (badge) => badge.offsetWidth > 0
    );
    const gap = Number.parseFloat(getComputedStyle(summaryElement).columnGap);
    const badgeGap = Number.isFinite(gap) ? gap : 0;
    const requiredWidth = visibleRequiredBadges.reduce(
      (width, badge) => width + badge.getBoundingClientRect().width,
      0
    );
    const availableWidth = summaryElement.getBoundingClientRect().width;

    let visibleBadgeCount = visibleRequiredBadges.length;
    let nextWidth = requiredWidth;

    const apiFits = fitsBadge(
      nextWidth,
      visibleBadgeCount,
      apiBadgeWidth,
      badgeGap,
      availableWidth
    );

    if (apiFits) {
      nextWidth +=
        gapBeforeNextBadge(visibleBadgeCount, badgeGap) + apiBadgeWidth;
      visibleBadgeCount += 1;
    }

    const matchFits =
      apiFits &&
      fitsBadge(
        nextWidth,
        visibleBadgeCount,
        matchBadgeWidth,
        badgeGap,
        availableWidth
      );

    showApiBadge = apiFits;
    showMatchBadge = matchFits;
    isMeasuringOptionalBadges = false;
  }

  return {
    get summaryElement() {
      return summaryElement;
    },
    set summaryElement(nextElement: HTMLDivElement | null) {
      summaryElement = nextElement;
    },
    get apiBadgeElement() {
      return apiBadgeElement;
    },
    set apiBadgeElement(nextElement: HTMLSpanElement | null) {
      apiBadgeElement = nextElement;
    },
    get matchBadgeElement() {
      return matchBadgeElement;
    },
    set matchBadgeElement(nextElement: HTMLSpanElement | null) {
      matchBadgeElement = nextElement;
    },
    get showApiBadge() {
      return showApiBadge;
    },
    get showMatchBadge() {
      return showMatchBadge;
    },
    get isMeasuringOptionalBadges() {
      return isMeasuringOptionalBadges;
    }
  };
}

function fitsBadge(
  currentWidth: number,
  currentBadgeCount: number,
  badgeWidth: number,
  badgeGap: number,
  availableWidth: number
) {
  if (badgeWidth <= 0) {
    return false;
  }

  return (
    currentWidth +
      gapBeforeNextBadge(currentBadgeCount, badgeGap) +
      badgeWidth <=
    availableWidth
  );
}

function gapBeforeNextBadge(currentBadgeCount: number, badgeGap: number) {
  return currentBadgeCount > 0 ? badgeGap : 0;
}
