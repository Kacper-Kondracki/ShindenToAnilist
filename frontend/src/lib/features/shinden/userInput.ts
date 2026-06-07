const shindenProfilePattern =
  /^(?:https?:\/\/)?(?:www\.)?(?:shinden\.pl\/user\/)?(\d+)(?:-[A-Za-z0-9_-]+)?\/?$/;

const shindenProfileHostPattern =
  /^(?:https?:\/\/)?(?:www\.)?shinden\.pl\/user\/\d+(?:-[A-Za-z0-9_-]+)?\/?$/;

export function parseShindenUserId(value: string) {
  const query = value.trim();
  if (!query) return null;

  const match = query.match(shindenProfilePattern);
  if (!match) return null;

  const userId = Number(match[1]);
  return Number.isSafeInteger(userId) && userId > 0 ? userId : null;
}

export function hasShindenProfileHost(value: string) {
  return shindenProfileHostPattern.test(value.trim());
}
