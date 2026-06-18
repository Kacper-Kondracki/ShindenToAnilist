import type { Provider } from '../../config/providers';
import { SourceProvider } from '../../gen/shinden_to_anilist/v1/common_pb';
import {
  hasShindenProfileHost,
  parseShindenUserId
} from '../shinden/userInput';

const animeZoneUserPattern =
  /^(?:https?:\/\/)?(?:www\.)?animezone\.pl\/user\/([^/?#]+)(?:\/(?:rated|watching|plans))?\/?(?:[?#].*)?$/i;
const sourceImportPreviewInput = 'shindentoanilist:source-import-preview';

const usernamePattern = /^[A-Za-z0-9_-]+$/;

export type ParsedSourceUser = {
  provider: SourceProvider;
  user: string;
  manualOverrideScopeKey: string;
};

export function parseSourceUser(provider: Provider, value: string) {
  if (provider === 'shinden') {
    const userId = parseShindenUserId(value);

    return userId === null
      ? null
      : {
          provider: SourceProvider.SHINDEN,
          user: String(userId),
          manualOverrideScopeKey: `${provider}:${userId}`
        };
  }

  if (provider === 'anime-zone') {
    const username = parseAnimeZoneUsername(value);

    return username === null
      ? null
      : {
          provider: SourceProvider.ANIME_ZONE,
          user: username,
          manualOverrideScopeKey: `${provider}:${username.toLowerCase()}`
        };
  }

  return null;
}

export function providerFromInput(value: string): Provider | null {
  if (isSourceImportPreviewInput(value)) {
    return 'anime-zone';
  }

  if (hasShindenProfileHost(value)) {
    return 'shinden';
  }

  if (animeZoneUserPattern.test(value.trim())) {
    return 'anime-zone';
  }

  return null;
}

export function isSourceImportPreviewInput(value: string) {
  return value === sourceImportPreviewInput;
}

function parseAnimeZoneUsername(value: string) {
  const query = value.trim();
  if (!query) return null;

  const urlMatch = query.match(animeZoneUserPattern);
  const username = urlMatch?.[1] ?? query;

  return usernamePattern.test(username) ? username : null;
}
