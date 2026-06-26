import type { Provider } from '../../config/providers';

const sourceImportPreviewInput = 'shindentoanilist:source-import-preview';
const shindenCloudflareAutoCloseTestInput =
  'shindentoanilist:shinden-cf-autoclose-test';
const showNotificationsPattern = /^shindentoanilist:show-notifications:(\d+)$/;

export type SourceDevCommand =
  | { kind: 'sourceImportPreview' }
  | { kind: 'shindenCloudflareAutoCloseTest' }
  | { kind: 'showMockNotifications'; count: number };

export function parseSourceDevCommand(value: string): SourceDevCommand | null {
  const query = value.trim();

  if (query === sourceImportPreviewInput) {
    return { kind: 'sourceImportPreview' };
  }

  if (query === shindenCloudflareAutoCloseTestInput) {
    return { kind: 'shindenCloudflareAutoCloseTest' };
  }

  const notificationMatch = query.match(showNotificationsPattern);
  if (notificationMatch !== null) {
    const count = Number(notificationMatch[1]);

    return Number.isSafeInteger(count) && count > 0
      ? { kind: 'showMockNotifications', count }
      : null;
  }

  return null;
}

export function providerForDevCommand(value: string): Provider | null {
  const command = parseSourceDevCommand(value);

  if (command?.kind === 'sourceImportPreview') {
    return 'anime-zone';
  }

  if (command?.kind === 'shindenCloudflareAutoCloseTest') {
    return 'shinden';
  }

  return null;
}
