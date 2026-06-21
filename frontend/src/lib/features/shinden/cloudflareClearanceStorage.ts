import type { ShindenCloudflareClearance } from '../../domain/anime';

const storageKey = 'shinden-to-anilist:shinden-cloudflare-clearance:v1';

export function readShindenCloudflareClearance() {
  const storage = getLocalStorage();
  if (storage === null) {
    return null;
  }

  try {
    const clearance = normalizeClearance(
      JSON.parse(storage.getItem(storageKey) ?? 'null')
    );

    if (clearance === null || isExpired(clearance)) {
      storage.removeItem(storageKey);
      return null;
    }

    return clearance;
  } catch {
    storage.removeItem(storageKey);
    return null;
  }
}

export function writeShindenCloudflareClearance(
  clearance: ShindenCloudflareClearance
) {
  const storage = getLocalStorage();
  if (storage === null || isExpired(clearance)) {
    return;
  }

  try {
    storage.setItem(storageKey, JSON.stringify(clearance));
  } catch {
    // Persisting Cloudflare clearance is best-effort; the active import already has it in memory.
  }
}

export function clearShindenCloudflareClearance() {
  const storage = getLocalStorage();
  if (storage === null) {
    return;
  }

  try {
    storage.removeItem(storageKey);
  } catch {
    // Clearing storage is best-effort.
  }
}

function normalizeClearance(value: unknown): ShindenCloudflareClearance | null {
  if (!isRecord(value)) {
    return null;
  }

  const userAgent = normalizeNonEmptyString(value.userAgent);
  const cfClearance = normalizeNonEmptyString(value.cfClearance);
  const domain = normalizeNonEmptyString(value.domain) ?? 'lista.shinden.pl';
  const path = normalizePath(value.path);
  const capturedAtMs = normalizeTimestampMs(value.capturedAtMs);
  const expiresUnixSeconds = normalizeOptionalNumber(value.expiresUnixSeconds);

  if (userAgent === null || cfClearance === null || capturedAtMs === null) {
    return null;
  }

  return {
    userAgent,
    cfClearance,
    domain,
    path,
    ...(expiresUnixSeconds === undefined ? {} : { expiresUnixSeconds }),
    capturedAtMs
  };
}

function isExpired(clearance: ShindenCloudflareClearance) {
  return (
    clearance.expiresUnixSeconds !== undefined &&
    clearance.expiresUnixSeconds <= Date.now() / 1000
  );
}

function normalizeNonEmptyString(value: unknown) {
  if (typeof value !== 'string') {
    return null;
  }

  const normalized = value.trim();
  return normalized === '' ? null : normalized;
}

function normalizePath(value: unknown) {
  const path = normalizeNonEmptyString(value);
  return path?.startsWith('/') === true ? path : '/';
}

function normalizeTimestampMs(value: unknown) {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0
    ? value
    : null;
}

function normalizeOptionalNumber(value: unknown) {
  return typeof value === 'number' && Number.isFinite(value)
    ? value
    : undefined;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function getLocalStorage() {
  return typeof localStorage === 'undefined' ? null : localStorage;
}
