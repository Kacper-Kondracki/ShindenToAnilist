<script lang="ts">
  import type { Snippet } from 'svelte';
  import {
    openContextMenu,
    type ContextMenuItem
  } from './contextMenuState.svelte';

  export type { ContextMenuItem } from './contextMenuState.svelte';

  let {
    items,
    ariaLabel = 'Menu kontekstowe',
    children,
    class: className = ''
  }: {
    items: ContextMenuItem[];
    ariaLabel?: string;
    children?: Snippet;
    class?: string;
  } = $props();

  let hasEnabledItems = $derived(items.some((item) => !item.disabled));

  function openAt(x: number, y: number) {
    openContextMenu({
      items,
      ariaLabel,
      x,
      y
    });
  }

  function handleContextMenu(event: MouseEvent) {
    if (!hasEnabledItems) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    openAt(event.clientX, event.clientY);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (
      event.key !== 'ContextMenu' &&
      !(event.shiftKey && event.key === 'F10')
    ) {
      return;
    }

    if (!hasEnabledItems) {
      return;
    }

    const target = event.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();

    event.preventDefault();
    event.stopPropagation();
    openAt(rect.left + 24, rect.top + Math.min(rect.height - 8, 48));
  }
</script>

<div
  class={`context-menu-host ${className}`}
  role="presentation"
  oncontextmenu={handleContextMenu}
  onkeydown={handleKeydown}
>
  {@render children?.()}
</div>

<style>
  .context-menu-host {
    display: block;
    width: 100%;
    min-width: 0;
  }
</style>
