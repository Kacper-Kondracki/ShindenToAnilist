import type { Date as ProtoDate } from '../gen/shinden_to_anilist/v1/common_pb';
import type { WireNumber } from '../domain/anime';

export type WireNumberInput = WireNumber | number | string;

export type TauriDate = {
  year: number;
  month: number;
  day: number;
};

export type WireDate = ProtoDate | TauriDate | null | undefined;

const U64_MAX = (1n << 64n) - 1n;

export function toWireNumber(value: WireNumberInput): WireNumber {
  if (typeof value === 'number') {
    if (!Number.isInteger(value)) {
      throw new Error(`Id lub wersja nie jest liczbą całkowitą: ${value}`);
    }

    if (!Number.isSafeInteger(value)) {
      throw new Error(
        `Id lub wersja poza bezpiecznym zakresem number: ${value}`
      );
    }

    if (value < 0) {
      throw new Error(`Id lub wersja poza zakresem u64: ${value}`);
    }

    return BigInt(value);
  }

  if (typeof value === 'string') {
    if (!/^\d+$/.test(value)) {
      throw new Error(`Id lub wersja nie jest liczbą całkowitą: ${value}`);
    }

    value = BigInt(value);
  }

  if (value < 0n || value > U64_MAX) {
    throw new Error(`Id lub wersja poza zakresem u64: ${value}`);
  }

  return value;
}

export function toTauriWireNumber(value: WireNumberInput): string {
  return toWireNumber(value).toString();
}

export function toSafeNumber(value: WireNumberInput): number {
  const normalized = toWireNumber(value);

  const numberValue = Number(normalized);
  if (!Number.isSafeInteger(numberValue)) {
    throw new Error(`Id lub wersja poza bezpiecznym zakresem number: ${value}`);
  }

  return numberValue;
}

export function formatProtoDate(date: WireDate) {
  if (date == null) {
    return null;
  }

  const month = String(date.month).padStart(2, '0');
  const day = String(date.day).padStart(2, '0');
  return `${date.year}-${month}-${day}`;
}

export function isGreaterWireNumber(left: WireNumber, right: WireNumber) {
  return left > right;
}
