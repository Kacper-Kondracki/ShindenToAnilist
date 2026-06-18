import { untrack } from 'svelte';

import type { SourceImportProgress as SourceImportProgressState } from '../../domain/anime';
import { SourceFetchPhase } from '../../gen/shinden_to_anilist/v1/source_pb';

const recentTitleCount = 5;

type RecentTitle = {
  id: number;
  title: string;
  tone: string;
};

type RecentTitleRow = RecentTitle & {
  gridRow: number;
};

type SourceImportProgressControllerInput = {
  getProviderLabel: () => string;
  getProgress: () => SourceImportProgressState | null;
};

export type SourceImportProgressController = ReturnType<
  typeof createSourceImportProgressController
>;

export function createSourceImportProgressController(
  input: SourceImportProgressControllerInput
) {
  let now = $state(performance.now());
  let recentTitles = $state<RecentTitle[]>([]);
  let lastProgressKey = '';
  let lastRecordedStep = -1;
  let lastRecordedTitle = '';
  let recentTitleId = 0;

  let progress = $derived.by(() => input.getProgress());
  let progressPercent = $derived(getProgressPercent(progress));
  let elapsedSeconds = $derived(
    progress === null
      ? 0
      : Math.max(0, Math.floor((now - progress.startedAt) / 1000))
  );
  let phaseText = $derived(translatePhase(progress?.phase));
  let progressCountText = $derived(getProgressCountText(progress));
  let progressTone = $derived(getProgressTone(progressPercent));
  let elapsedText = $derived(formatElapsed(elapsedSeconds));
  let recentTitleRows = $derived.by((): RecentTitleRow[] =>
    recentTitles.slice(-recentTitleCount).map((row, index, rows) => ({
      ...row,
      gridRow: recentTitleCount - rows.length + index + 1
    }))
  );
  let hasRecentTitles = $derived(recentTitles.length > 0);

  $effect(() => {
    const interval = window.setInterval(() => {
      now = performance.now();
    }, 500);

    return () => window.clearInterval(interval);
  });

  $effect(() => {
    if (progress === null) {
      resetRecentTitles();
      return;
    }

    const progressKey = `${input.getProviderLabel()}:${progress.startedAt}`;

    if (progressKey !== lastProgressKey) {
      resetRecentTitles(progressKey);
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
          tone: getProgressTone(getProgressPercent(progress))
        }
      ].slice(-recentTitleCount)
    );
    lastRecordedStep = progress.current;
    lastRecordedTitle = latestTitle;
  });

  function resetRecentTitles(nextProgressKey = '') {
    recentTitles = [];
    lastProgressKey = nextProgressKey;
    lastRecordedStep = -1;
    lastRecordedTitle = '';
    recentTitleId = 0;
  }

  return {
    get progressPercent() {
      return progressPercent;
    },
    get phaseText() {
      return phaseText;
    },
    get progressCountText() {
      return progressCountText;
    },
    get progressTone() {
      return progressTone;
    },
    get elapsedText() {
      return elapsedText;
    },
    get recentTitleRows() {
      return recentTitleRows;
    },
    get hasRecentTitles() {
      return hasRecentTitles;
    }
  };
}

function getProgressPercent(progress: SourceImportProgressState | null) {
  if (progress === null || progress.total === 0) {
    return 0;
  }

  return Math.min(100, Math.round((progress.current / progress.total) * 100));
}

function getProgressCountText(progress: SourceImportProgressState | null) {
  return progress === null || progress.total === 0
    ? 'Oczekiwanie na dane'
    : `${progress.current} / ${progress.total}`;
}

function getProgressTone(progressPercent: number) {
  return `color-mix(in oklab, var(--ctp-mocha-red) ${100 - progressPercent}%, var(--ctp-mocha-green) ${progressPercent}%)`;
}

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
