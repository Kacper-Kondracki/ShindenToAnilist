import { providers, type Provider } from '../../config/providers';
import type { SourceImportProgress } from '../../domain/anime';
import { SourceProvider } from '../../gen/shinden_to_anilist/v1/common_pb';
import { SourceFetchPhase } from '../../gen/shinden_to_anilist/v1/source_pb';
import {
  hasShindenProfileHost,
  parseShindenUserId
} from '../shinden/userInput';
import { providerForDevCommand } from './devCommands';

export type ParsedSourceUser = {
  provider: SourceProvider;
  user: string;
  manualOverrideScopeKey: string;
};

type SourceProviderProgressPolicy = {
  debounceListPhaseMs?: number;
  isReady?: (
    progress: SourceImportProgress,
    options: { listPhaseDebounced: boolean }
  ) => boolean;
};

type SourceProviderBehavior = {
  id: Provider;
  wireProvider: SourceProvider;
  parseUser: (value: string) => ParsedSourceUser | null;
  hasExplicitUserUrl: (value: string) => boolean;
  progress: SourceProviderProgressPolicy;
};

const animeZoneUserPattern =
  /^(?:https?:\/\/)?(?:www\.)?animezone\.pl\/user\/([^/?#]+)(?:\/(?:rated|watching|plans))?\/?(?:[?#].*)?$/i;
const ogladajAnimeUserPattern =
  /^(?:https?:\/\/)?(?:www\.)?ogladajanime\.pl\/(?:anime_list|profile)\/(\d+)(?:\/[0-5])?\/?(?:[?#].*)?$/i;
const usernamePattern = /^[A-Za-z0-9_-]+$/;
const numericUserIdPattern = /^\d+$/;

const defaultProgressPolicy: SourceProviderProgressPolicy = {
  isReady: (progress) =>
    progress.total > 0 || progress.phase !== SourceFetchPhase.FETCHING_LIST
};

const sourceProviderBehaviors = [
  {
    id: 'shinden',
    wireProvider: SourceProvider.SHINDEN,
    parseUser: (value) => {
      const userId = parseShindenUserId(value);

      return userId === null
        ? null
        : {
            provider: SourceProvider.SHINDEN,
            user: String(userId),
            manualOverrideScopeKey: `shinden:${userId}`
          };
    },
    hasExplicitUserUrl: hasShindenProfileHost,
    progress: defaultProgressPolicy
  },
  {
    id: 'ogladaj-anime',
    wireProvider: SourceProvider.OGLADAJ_ANIME,
    parseUser: (value) => {
      const userId = parseOgladajAnimeUserId(value);

      return userId === null
        ? null
        : {
            provider: SourceProvider.OGLADAJ_ANIME,
            user: userId,
            manualOverrideScopeKey: `ogladaj-anime:${userId}`
          };
    },
    hasExplicitUserUrl: (value) => ogladajAnimeUserPattern.test(value.trim()),
    progress: defaultProgressPolicy
  },
  {
    id: 'anime-zone',
    wireProvider: SourceProvider.ANIME_ZONE,
    parseUser: (value) => {
      const username = parseAnimeZoneUsername(value);

      return username === null
        ? null
        : {
            provider: SourceProvider.ANIME_ZONE,
            user: username,
            manualOverrideScopeKey: `anime-zone:${username.toLowerCase()}`
          };
    },
    hasExplicitUserUrl: (value) => animeZoneUserPattern.test(value.trim()),
    progress: {
      debounceListPhaseMs: 250,
      isReady: (progress, options) =>
        progress.total > 0 ||
        progress.phase !== SourceFetchPhase.FETCHING_LIST ||
        options.listPhaseDebounced
    }
  }
] as const satisfies readonly SourceProviderBehavior[];

export function sourceProviderBehaviorById(provider: Provider) {
  return (
    sourceProviderBehaviors.find(({ id }) => id === provider) ??
    sourceProviderBehaviors[0]
  );
}

export function parseSourceUser(provider: Provider, value: string) {
  return sourceProviderBehaviorById(provider).parseUser(value);
}

export function providerFromInput(value: string): Provider | null {
  const devCommandProvider = providerForDevCommand(value);
  if (devCommandProvider !== null) {
    return devCommandProvider;
  }

  return (
    sourceProviderBehaviors.find((provider) =>
      provider.hasExplicitUserUrl(value)
    )?.id ?? null
  );
}

export function isReadySourceImportProgress(
  provider: Provider | string,
  progress: SourceImportProgress | null,
  options: { listPhaseDebounced?: boolean } = {}
) {
  if (progress === null || !isProvider(provider)) {
    return false;
  }

  const policy = sourceProviderBehaviorById(provider).progress;
  return (
    policy.isReady?.(progress, {
      listPhaseDebounced: options.listPhaseDebounced ?? false
    }) ?? true
  );
}

export function shouldDebounceSourceImportProgress(
  provider: Provider,
  progress: SourceImportProgress | null
) {
  if (progress === null) {
    return false;
  }

  const policy = sourceProviderBehaviorById(provider).progress;
  return (
    policy.debounceListPhaseMs !== undefined &&
    progress.total === 0 &&
    progress.phase === SourceFetchPhase.FETCHING_LIST
  );
}

export function sourceImportProgressDebounceMs(provider: Provider) {
  return sourceProviderBehaviorById(provider).progress.debounceListPhaseMs ?? 0;
}

function parseAnimeZoneUsername(value: string) {
  const query = value.trim();
  if (!query) return null;

  const urlMatch = query.match(animeZoneUserPattern);
  const username = urlMatch?.[1] ?? query;

  return usernamePattern.test(username) ? username : null;
}

function parseOgladajAnimeUserId(value: string) {
  const query = value.trim();
  if (!query) return null;

  const urlMatch = query.match(ogladajAnimeUserPattern);
  const userId = urlMatch?.[1] ?? query;

  return numericUserIdPattern.test(userId) ? userId : null;
}

function isProvider(value: string): value is Provider {
  return providers.some(({ id }) => id === value);
}
