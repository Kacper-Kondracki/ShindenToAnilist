<script lang="ts">
  import type { Provider, ProviderOption } from '../config/providers';
  import type { DatabaseState } from '../domain/anime';
  import DatabaseStatus from './DatabaseStatus.svelte';
  import ProviderSelector from './ProviderSelector.svelte';
  import UserListForm from './UserListForm.svelte';

  let {
    providers,
    selectedProvider,
    userQuery,
    databaseState,
    databaseStatusText,
    isUserListLoading,
    isLoadButtonBusy,
    hasUserListError,
    userListErrorMessage,
    canSubmit,
    onSelectProvider,
    onUserQueryInput,
    onClearUserListError,
    onSubmit
  }: {
    providers: readonly ProviderOption[];
    selectedProvider: Provider;
    userQuery: string;
    databaseState: DatabaseState;
    databaseStatusText: string;
    isUserListLoading: boolean;
    isLoadButtonBusy: boolean;
    hasUserListError: boolean;
    userListErrorMessage: string | null;
    canSubmit: boolean;
    onSelectProvider: (provider: Provider) => void;
    onUserQueryInput: (value: string) => void;
    onClearUserListError: () => void;
    onSubmit: (event: SubmitEvent) => void;
  } = $props();
</script>

<header class="border-base-content/10 bg-base-200 border-b">
  <div class="flex flex-wrap items-center gap-4 px-4 py-3">
    <div
      class="flex flex-[0_0_max-content] flex-nowrap items-center justify-start max-[58rem]:flex-[1_0_100%] max-[58rem]:justify-between"
    >
      <div class="min-w-52">
        <h1 class="text-xl font-bold">ShindenToAnilist</h1>
        <DatabaseStatus state={databaseState} text={databaseStatusText} />
      </div>

      <ProviderSelector
        {providers}
        {selectedProvider}
        locked={isLoadButtonBusy}
        {onSelectProvider}
      />
    </div>

    <UserListForm
      value={userQuery}
      busy={isLoadButtonBusy}
      readonly={isLoadButtonBusy}
      {canSubmit}
      hasError={hasUserListError}
      errorMessage={userListErrorMessage}
      onValueInput={onUserQueryInput}
      onClearError={onClearUserListError}
      {onSubmit}
    />
  </div>
</header>
