import type { DatabaseEntry } from '../../domain/anime';

export async function copyText(text: string) {
  if (navigator.clipboard !== undefined && window.isSecureContext) {
    await navigator.clipboard.writeText(text);
    return;
  }

  const textarea = document.createElement('textarea');

  textarea.value = text;
  textarea.style.position = 'fixed';
  textarea.style.opacity = '0';
  textarea.setAttribute('readonly', '');
  document.body.appendChild(textarea);
  textarea.select();

  try {
    document.execCommand('copy');
  } finally {
    textarea.remove();
  }
}

export function openExternalUrl(url: string) {
  const openExternalUrl = globalThis.shindenToAnilist?.openExternalUrl;

  if (openExternalUrl !== undefined) {
    void openExternalUrl(url);
    return;
  }

  window.open(url, '_blank', 'noopener,noreferrer');
}

export function shindenEntryUrl(entryId: number) {
  return `https://shinden.pl/series/${entryId}`;
}

export function databaseEntryMalUrl(entry: DatabaseEntry) {
  return (
    entry.sources.find((source) => isMalAnimeUrl(source)) ??
    `https://myanimelist.net/anime/${entry.id}`
  );
}

function isMalAnimeUrl(source: string) {
  try {
    const url = new URL(source);

    return (
      url.protocol === 'https:' &&
      url.hostname.endsWith('myanimelist.net') &&
      url.pathname.startsWith('/anime/')
    );
  } catch {
    return false;
  }
}
