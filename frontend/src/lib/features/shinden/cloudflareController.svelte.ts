import type { Provider } from '../../config/providers';
import type { ShindenCloudflareClearance } from '../../domain/anime';
import type { ParsedSourceUser } from '../source/userInput';

export type ShindenCloudflareRequest = {
  provider: Provider;
  query: string;
  sourceUser: ParsedSourceUser;
  message: string;
};

export type ShindenCloudflareState =
  | { status: 'idle' }
  | ({ status: 'blocked' } & ShindenCloudflareRequest)
  | ({ status: 'openingWindow' } & ShindenCloudflareRequest)
  | ({
      status: 'applyingClearance';
      clearance: ShindenCloudflareClearance;
    } & ShindenCloudflareRequest)
  | ({ status: 'retrying' } & ShindenCloudflareRequest)
  | ({ status: 'failed' } & ShindenCloudflareRequest);

type ShindenCloudflareControllerInput = {
  openVerification: () => Promise<ShindenCloudflareClearance>;
  applyClearance: (
    clearance: ShindenCloudflareClearance
  ) => Promise<{ accepted: boolean }>;
  retry: (request: ShindenCloudflareRequest) => Promise<void>;
  onResolved?: () => void;
  onError?: (message: string) => void;
};

export function createShindenCloudflareController(
  input: ShindenCloudflareControllerInput
) {
  let state = $state<ShindenCloudflareState>({ status: 'idle' });
  let isBusy = $derived(
    state.status === 'openingWindow' ||
      state.status === 'applyingClearance' ||
      state.status === 'retrying'
  );
  let isOpen = $derived(state.status !== 'idle');

  function block(request: ShindenCloudflareRequest) {
    state = { ...request, status: 'blocked' };
  }

  function cancel() {
    if (isBusy) {
      return;
    }

    state = { status: 'idle' };
  }

  async function resolve() {
    if (state.status !== 'blocked' && state.status !== 'failed') {
      return;
    }

    const request = {
      provider: state.provider,
      query: state.query,
      sourceUser: state.sourceUser,
      message: state.message
    };

    try {
      state = { ...request, status: 'openingWindow' };
      const clearance = await input.openVerification();
      state = { ...request, status: 'applyingClearance', clearance };
      const applied = await input.applyClearance(clearance);

      if (!applied.accepted) {
        throw new Error('Nie udało się zastosować ciasteczka Cloudflare.');
      }

      state = { ...request, status: 'retrying' };
      await input.retry(request);
      state = { status: 'idle' };
      input.onResolved?.();
    } catch (error) {
      const message = errorToMessage(error);
      state = { ...request, status: 'failed', message };
      input.onError?.(message);
    }
  }

  return {
    get state() {
      return state;
    },
    get isBusy() {
      return isBusy;
    },
    get isOpen() {
      return isOpen;
    },
    block,
    cancel,
    resolve
  };
}

function errorToMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  return 'Nie udało się zakończyć weryfikacji Cloudflare.';
}
