<script lang="ts">
  import type { Provider, ProviderOption } from "../config/providers";
  import type { DatabaseState } from "../domain/anime";
  import DatabaseStatus from "./DatabaseStatus.svelte";
  import ProviderSelector from "./ProviderSelector.svelte";
  import UserListForm from "./UserListForm.svelte";

  let {
    providers,
    selectedProvider = $bindable(),
    userQuery = $bindable(),
    databaseState,
    databaseStatusText,
    isLoadButtonBusy,
    hasUserListError,
    userListErrorMessage,
    canSubmit,
    onClearUserListError,
    onSubmit,
  }: {
    providers: readonly ProviderOption[];
    selectedProvider: Provider;
    userQuery: string;
    databaseState: DatabaseState;
    databaseStatusText: string;
    isLoadButtonBusy: boolean;
    hasUserListError: boolean;
    userListErrorMessage?: string;
    canSubmit: boolean;
    onClearUserListError: () => void;
    onSubmit: (event: SubmitEvent) => void;
  } = $props();
</script>

<header class="app-header">
  <div class="app-header-body">
    <div class="app-header-primary">
      <div class="min-w-52">
        <h1 class="text-xl font-bold">ShindenToAnilist</h1>
        <DatabaseStatus state={databaseState} text={databaseStatusText} />
      </div>

      <ProviderSelector {providers} bind:selectedProvider />
    </div>

    <UserListForm
      bind:value={userQuery}
      busy={isLoadButtonBusy}
      {canSubmit}
      hasError={hasUserListError}
      errorMessage={userListErrorMessage}
      onClearError={onClearUserListError}
      {onSubmit}
    />
  </div>
</header>
