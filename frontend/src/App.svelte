<script lang="ts">
  import { onMount } from 'svelte';

  import AppHeader from './lib/components/AppHeader.svelte';
  import EmptyWorkspace from './lib/components/EmptyWorkspace.svelte';
  import NotificationLayer from './lib/components/NotificationLayer.svelte';
  import ShindenCloudflareChallengeModal from './lib/components/ShindenCloudflareChallengeModal.svelte';
  import WorkspaceView from './lib/components/WorkspaceView.svelte';
  import { createAppController } from './lib/features/app/appController.svelte';

  const app = createAppController();
  let enableViewEnterAnimations = $state(false);

  onMount(() => {
    void app.initializeDatabase();
  });

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    enableViewEnterAnimations = true;
    await app.submitUserList();
  }
</script>

<main
  class="bg-base-300 flex min-h-dvh flex-col"
  style:--provider-accent={app.selectedProviderDetails.accent}
>
  <AppHeader
    providers={app.providers}
    selectedProvider={app.selectedProvider}
    userQuery={app.userQuery}
    databaseState={app.databaseState}
    databaseStatusText={app.databaseStatusText}
    isUserListLoading={app.isUserListLoading}
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
    {#if app.workspace.state.status === 'empty' || app.shouldShowSourceImportProgress}
      <div
        class="view-frame"
        class:view-frame--enter={enableViewEnterAnimations &&
          app.shouldShowSourceImportProgress}
      >
        {#key app.shouldShowSourceImportProgress}
          <EmptyWorkspace
            provider={app.userListRequestProviderDetails}
            animateOnMount={enableViewEnterAnimations}
            canLoadProvider={app.isProviderSupported}
            sourceImportProgressDebounced={app.isSourceImportProgressDebounced}
            userListRequestState={app.userListRequestState}
            onCancelLoad={app.cancelUserListLoad}
          />
        {/key}
      </div>
    {:else}
      <div
        class="view-frame"
        class:view-frame--enter={enableViewEnterAnimations}
      >
        {#key app.workspace.state}
          <WorkspaceView
            providerLabel={app.activeProviderDetails.label}
            animeData={app.animeData}
            workspace={app.workspace}
          />
        {/key}
      </div>
    {/if}
  </div>

  <NotificationLayer controller={app.notifications} />
  <ShindenCloudflareChallengeModal
    state={app.shindenCloudflare.state}
    busy={app.shindenCloudflare.isBusy}
    onCancel={app.shindenCloudflare.cancel}
    onOpenVerification={app.shindenCloudflare.resolve}
  />
</main>

<style>
  .view-frame {
    display: flex;
    position: absolute;
    inset: 0;
    min-height: 0;
    flex-direction: column;
  }

  .view-frame--enter {
    animation: view-enter 600ms cubic-bezier(0.22, 1, 0.36, 1) both;
    backface-visibility: hidden;
    transform: translateZ(0);
    will-change: transform, opacity;
  }

  @keyframes view-enter {
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
    .view-frame--enter {
      animation: none;
    }
  }
</style>
