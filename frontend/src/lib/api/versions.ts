import type { WireNumber } from '../domain/anime';
import {
  isGreaterWireNumber,
  toWireNumber,
  type WireNumberInput
} from './wire';

let shindenVersion: WireNumber = 0n;
let sourceVersion: WireNumber = 0n;
let databaseVersion: WireNumber = 0n;

export function currentVersions() {
  return {
    source: sourceVersion,
    shinden: shindenVersion,
    database: databaseVersion
  };
}

export function currentSourceVersion() {
  return sourceVersion;
}

export function observeShindenVersion(version: WireNumberInput) {
  const nextVersion = toWireNumber(version);
  if (isGreaterWireNumber(nextVersion, shindenVersion)) {
    shindenVersion = nextVersion;
  }
}

export function observeSourceVersion(version: WireNumberInput) {
  const nextVersion = toWireNumber(version);
  if (isGreaterWireNumber(nextVersion, sourceVersion)) {
    sourceVersion = nextVersion;
  }
  observeShindenVersion(version);
}

export function observeDatabaseVersion(version: WireNumberInput) {
  const nextVersion = toWireNumber(version);
  if (isGreaterWireNumber(nextVersion, databaseVersion)) {
    databaseVersion = nextVersion;
  }
}
