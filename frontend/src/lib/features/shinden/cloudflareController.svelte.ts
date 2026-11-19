import { reportFailure } from '../../api/diagnostics';
import {
  AppFailure,
  ErrorKind,
  isFailure,
  userErrorMessage
} from '../../api/errors';
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
  saveClearance?: (clearance: ShindenCloudflareClearance) => void;
  retry: (
    request: ShindenCloudflareRequest
  ) => Promise<'loaded' | 'cancelled' | 'failed' | 'challenge'>;
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
        throw new AppFailure(ErrorKind.CLEARANCE_INVALID);
      }

      state = { ...request, status: 'retrying' };
      const result = await input.retry(request);
      if (result === 'challenge') {
        state = {
          ...request,
          status: 'failed',
          message:
            'Shinden nadal wymaga weryfikacji. Otwórz okno i dokończ weryfikację na stronie.'
        };
        return;
      }

      if (result === 'cancelled') {
        state = { status: 'idle' };
        return;
      }
      input.saveClearance?.(clearance);
      state = { status: 'idle' };
    } catch (error) {
      if (isFailure(error, ErrorKind.CANCELLED)) {
        state = {
          ...request,
          status: 'blocked',
          message:
            'Weryfikacja została przerwana. Otwórz okno ponownie, aby kontynuować.'
        };
        return;
      }

      reportFailure('shinden_verification', error);
      state = {
        ...request,
        status: 'failed',
        message: userErrorMessage(
          error,
          'Nie udało się zakończyć weryfikacji Shindena.'
        )
      };
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
