<script lang="ts">
  import { cubicOut } from 'svelte/easing';
  import { fly } from 'svelte/transition';

  import type {
    AppNotification,
    NotificationController,
    NotificationTone
  } from '../features/notifications/notificationController.svelte';

  let {
    controller
  }: {
    controller: NotificationController;
  } = $props();

  function toneClass(tone: NotificationTone) {
    return {
      info: 'notification-entry--info',
      success: 'notification-entry--success',
      warning: 'notification-entry--warning',
      error: 'notification-entry--error'
    }[tone];
  }

  function iconClass(tone: NotificationTone) {
    return {
      info: 'icon-[lucide--info]',
      success: 'icon-[lucide--circle-check]',
      warning: 'icon-[lucide--triangle-alert]',
      error: 'icon-[lucide--circle-alert]'
    }[tone];
  }

  function role(notification: AppNotification) {
    return notification.tone === 'error' || notification.tone === 'warning'
      ? 'alert'
      : 'status';
  }

  function dismissFromButton(event: MouseEvent, notificationId: number) {
    event.stopPropagation();
    controller.dismiss(notificationId);
  }
</script>

<div
  class="toast toast-bottom toast-end pointer-events-none z-50 w-full max-w-sm p-4"
  aria-live="polite"
>
  {#each controller.notifications as notification (notification.id)}
    <div
      class={`notification-entry alert ${toneClass(notification.tone)} bg-base-100/95 text-base-content pointer-events-auto grid grid-cols-[auto_1fr_auto] items-start gap-3 border-2 shadow-2xl backdrop-blur`}
      role={role(notification)}
      onclick={() => controller.dismiss(notification.id)}
      out:fly={{ x: 440, duration: 360, opacity: 0, easing: cubicOut }}
      onoutroend={() => controller.completeDismiss(notification.id)}
    >
      <span
        class={`${iconClass(notification.tone)} notification-entry__icon mt-0.5 size-5 shrink-0`}
        aria-hidden="true"
      ></span>
      <div class="min-w-0">
        <p class="text-sm font-bold">{notification.title}</p>
        <p class="text-sm leading-snug wrap-break-word opacity-90">
          {notification.message}
        </p>
      </div>
      <button
        class="btn btn-ghost btn-xs btn-square -m-1"
        type="button"
        aria-label="Zamknij powiadomienie"
        onclick={(event) => dismissFromButton(event, notification.id)}
      >
        <span class="icon-[lucide--x] size-4" aria-hidden="true"></span>
      </button>
    </div>
  {/each}
</div>

<style>
  .notification-entry {
    --notification-accent: var(--color-info);

    border-color: var(--notification-accent);
  }

  .notification-entry__icon {
    color: var(--notification-accent);
  }

  .notification-entry--info {
    --notification-accent: var(--color-info);
  }

  .notification-entry--success {
    --notification-accent: var(--color-success);
  }

  .notification-entry--warning {
    --notification-accent: var(--color-warning);
  }

  .notification-entry--error {
    --notification-accent: var(--color-error);
  }
</style>
