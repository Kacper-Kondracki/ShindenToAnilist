<script lang="ts">
  import { onMount } from 'svelte';

  import AppHeader from './lib/components/AppHeader.svelte';
  import EmptyWorkspace from './lib/components/EmptyWorkspace.svelte';
  import WorkspaceView from './lib/components/WorkspaceView.svelte';
  import { createAppController } from './lib/features/app/appController.svelte';

  const app = createAppController();

  onMount(() => {
    void app.initializeDatabase();
  });

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    await app.submitUserList();
  }
</script>

<main
  class="flex min-h-dvh flex-col bg-base-300"
  style:--provider-accent={app.selectedProviderDetails.accent}
>
  <AppHeader
    providers={app.providers}
    selectedProvider={app.selectedProvider}
    userQuery={app.userQuery}
    databaseState={app.databaseState}
    databaseStatusText={app.databaseStatusText}
    isLoadButtonBusy={app.isLoadButtonBusy}
    hasUserListError={app.hasUserListError}
    userListErrorMessage={app.userListErrorMessage}
    canSubmit={app.canSubmit}
    onSelectProvider={app.setSelectedProvider}
    onUserQueryInput={app.setUserQuery}
    onClearUserListError={app.clearUserListError}
    onSubmit={handleSubmit}
  />

  <div class="relative min-h-0 flex-1 overflow-hidden contain-[layout_paint]">
    {#if app.workspace.state.status === 'empty'}
      <div class="view-frame">
        <EmptyWorkspace
          provider={app.selectedProviderDetails}
          canLoadProvider={app.isProviderSupported}
        />
      </div>
    {:else}
      <div class="view-frame view-frame--workspace-enter">
        {#key app.workspace.state}
          <WorkspaceView
            providerLabel={app.activeProviderDetails.label}
            entryIdsByView={app.workspace.state.entryIdsByView}
            animeData={app.animeData}
            workspace={app.workspace}
          />
        {/key}
      </div>
    {/if}
  </div>
</main>

<style>
  .view-frame {
    display: flex;
    position: absolute;
    inset: 0;
    min-height: 0;
    flex-direction: column;
  }

  .view-frame--workspace-enter {
    animation: workspace-enter 600ms cubic-bezier(0.22, 1, 0.36, 1) both;
    backface-visibility: hidden;
    transform: translateZ(0);
    will-change: transform, opacity;
  }

  @keyframes workspace-enter {
    from {
      opacity: 0;
      transform: translate3d(0, 4rem, 0);
    }

    to {
      opacity: 1;
      transform: translate3d(0, 0, 0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .view-frame--workspace-enter {
      animation: none;
    }
  }
</style>
