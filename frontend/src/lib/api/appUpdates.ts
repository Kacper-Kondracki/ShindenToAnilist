import { version as installedVersion } from '../../../../package.json';

const repository = 'Kacper-Kondracki/ShindenToAnilist';
const latestReleaseUrl = `https://api.github.com/repos/${repository}/releases/latest`;
const versionPattern =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(-[\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*)?(?:\+[\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*)?$/;

export type AppUpdate = {
  version: string;
  url: string;
};

export async function checkAppUpdate(
  signal: AbortSignal
): Promise<AppUpdate | null> {
  try {
    const response = await fetch(latestReleaseUrl, {
      credentials: 'omit',
      referrerPolicy: 'no-referrer',
      signal
    });
    if (!response.ok) return null;

    const release: unknown = await response.json();
    if (
      typeof release !== 'object' ||
      release === null ||
      !('tag_name' in release) ||
      typeof release.tag_name !== 'string' ||
      !('draft' in release) ||
      release.draft !== false ||
      !('prerelease' in release) ||
      release.prerelease !== false
    ) {
      return null;
    }

    const version = release.tag_name.replace(/^(?:v3-|v)/, '');
    const latest = parseVersion(version);
    const installed = parseVersion(installedVersion);
    if (latest === null || latest.prerelease || installed === null) {
      return null;
    }

    const difference =
      latest.major - installed.major ||
      latest.minor - installed.minor ||
      latest.patch - installed.patch;
    if (difference < 0n || (difference === 0n && !installed.prerelease)) {
      return null;
    }

    return {
      version,
      url: `https://github.com/${repository}/releases/tag/${encodeURIComponent(release.tag_name)}`
    };
  } catch {
    return null;
  }
}

function parseVersion(version: string) {
  const match = versionPattern.exec(version);
  if (match === null || match[0] !== version) return null;

  const [, major = '0', minor = '0', patch = '0', prerelease] = match;
  return {
    major: BigInt(major),
    minor: BigInt(minor),
    patch: BigInt(patch),
    prerelease: prerelease !== undefined
  };
}
