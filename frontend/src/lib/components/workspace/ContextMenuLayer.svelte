<script lang="ts">
  import { tick } from 'svelte';
  import {
    closeContextMenu,
    contextMenu,
    positionContextMenu,
    type ContextMenuItem
  } from './contextMenuState.svelte';

  let menuElement = $state<HTMLUListElement | null>(null);

  function portal(node: HTMLElement) {
    document.body.appendChild(node);

    return {
      destroy() {
        node.remove();
      }
    };
  }

  $effect(() => {
    if (contextMenu.status !== 'open') {
      return;
    }

    const x = contextMenu.x;
    const y = contextMenu.y;
    const itemCount = contextMenu.items.length;

    void tick().then(() => {
      if (
        contextMenu.status === 'open' &&
        contextMenu.x === x &&
        contextMenu.y === y &&
        contextMenu.items.length === itemCount
      ) {
        positionMenu();
        menuElement?.focus();
      }
    });
  });

  function positionMenu() {
    if (menuElement === null) {
      return;
    }

    const menuRect = menuElement.getBoundingClientRect();
    const viewportPadding = 8;
    const nextX = Math.max(
      viewportPadding,
      Math.min(
        contextMenu.x,
        window.innerWidth - menuRect.width - viewportPadding
      )
    );
    const nextY = Math.max(
      viewportPadding,
      Math.min(
        contextMenu.y,
        window.innerHeight - menuRect.height - viewportPadding
      )
    );

    if (nextX !== contextMenu.x || nextY !== contextMenu.y) {
      positionContextMenu(nextX, nextY);
    }
  }

  function selectItem(item: ContextMenuItem) {
    const result = item.onSelect();
    closeContextMenu();
    void result;
  }

  function handleItemPointerdown(event: PointerEvent, item: ContextMenuItem) {
    if (event.button !== 0) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    if (item.disabled) {
      return;
    }

    selectItem(item);
  }

  function handleItemKeydown(event: KeyboardEvent, item: ContextMenuItem) {
    if (event.key !== 'Enter' && event.key !== ' ') {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    if (item.disabled || event.repeat) {
      return;
    }

    selectItem(item);
  }

  function handleItemClick(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
  }

  function handleMenuContextMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    event.stopPropagation();

    if (event.key === 'Escape') {
      event.preventDefault();
      closeContextMenu();
    }
  }

  function handleWindowClick() {
    closeContextMenu();
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && contextMenu.isVisible) {
      event.preventDefault();
      closeContextMenu();
    }
  }
</script>

<svelte:window
  onclick={handleWindowClick}
  onkeydown={handleWindowKeydown}
  onresize={closeContextMenu}
/>

{#if contextMenu.isVisible}
  <ul
    bind:this={menuElement}
    use:portal
    class:context-menu--closing={contextMenu.status === 'closing'}
    class="context-menu menu bg-base-100 rounded-box border-base-content/10 z-50 min-w-56 border p-1 shadow-xl"
    style={`left: ${contextMenu.x}px; top: ${contextMenu.y}px`}
    role="menu"
    tabindex="-1"
    aria-label={contextMenu.ariaLabel}
    onclick={(event) => event.stopPropagation()}
    oncontextmenu={handleMenuContextMenu}
    onkeydown={handleMenuKeydown}
  >
    {#each contextMenu.items as item (item.id)}
      {#if item.dividerBefore}
        <li class="context-menu__divider" aria-hidden="true"></li>
      {/if}
      <li role="none">
        <button
          type="button"
          role={item.checked === undefined ? 'menuitem' : 'menuitemcheckbox'}
          aria-checked={item.checked}
          aria-disabled={item.disabled}
          class:context-menu__item--danger={item.danger}
          disabled={item.disabled}
          onpointerdown={(event) => handleItemPointerdown(event, item)}
          onkeydown={(event) => handleItemKeydown(event, item)}
          onclick={handleItemClick}
        >
          <span
            aria-hidden="true"
            class={`size-4 shrink-0 ${item.checked ? 'icon-[lucide--check]' : (item.icon ?? '')}`}
          ></span>
          <span class="truncate">{item.label}</span>
        </button>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .context-menu {
    position: fixed;
    overflow: hidden;
    outline: none;
    transform-origin: top left;
    animation: context-menu-in 90ms ease-out;
  }

  .context-menu--closing {
    pointer-events: none;
    animation: context-menu-out 120ms ease-in forwards;
  }

  .context-menu button {
    display: grid;
    width: 100%;
    grid-template-columns: 1rem minmax(0, 1fr);
    align-items: center;
    gap: calc(var(--spacing) * 2);
    text-align: left;
  }

  .context-menu button:active {
    background-color: transparent;
    color: inherit;
  }

  .context-menu button:disabled {
    pointer-events: none;
    cursor: not-allowed;
    opacity: 0.42;
  }

  .context-menu__item--danger {
    color: var(--color-error);
  }

  .context-menu__divider {
    height: var(--border);
    margin-block: calc(var(--spacing) * 1);
    background-color: color-mix(
      in oklab,
      var(--color-base-content) 10%,
      transparent
    );
  }

  @keyframes context-menu-in {
    from {
      opacity: 0;
      transform: scale(0.98) translateY(-0.125rem);
    }

    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  @keyframes context-menu-out {
    from {
      opacity: 1;
      transform: scale(1) translateY(0);
    }

    to {
      opacity: 0;
      transform: scale(0.98) translateY(-0.125rem);
    }
  }
</style>
