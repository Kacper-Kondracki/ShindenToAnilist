<script lang="ts">
  import type { Snippet } from 'svelte';

  export type EntryRowTone =
    | 'matched'
    | 'review'
    | 'unmatched'
    | 'neutral'
    | 'info'
    | 'suppressed'
    | 'ignored';

  let {
    tone = 'neutral',
    isSelected,
    ariaLabel,
    title,
    onSelect,
    showIndicator = true,
    rounded = false,
    compact = false,
    softWarning = false,
    children,
    meta,
    class: className = ''
  }: {
    tone?: EntryRowTone;
    isSelected: boolean;
    ariaLabel: string;
    title: string;
    onSelect: () => void;
    showIndicator?: boolean;
    rounded?: boolean;
    compact?: boolean;
    softWarning?: boolean;
    children?: Snippet;
    meta?: Snippet;
    class?: string;
  } = $props();
</script>

<button
  type="button"
  class:entry-row--matched={tone === 'matched'}
  class:entry-row--review={tone === 'review'}
  class:entry-row--unmatched={tone === 'unmatched'}
  class:entry-row--neutral={tone === 'neutral'}
  class:entry-row--info={tone === 'info'}
  class:entry-row--suppressed={tone === 'suppressed'}
  class:entry-row--ignored={tone === 'ignored'}
  class:entry-row--soft-warning={softWarning}
  class:entry-row--selected={isSelected}
  class:entry-row--without-indicator={!showIndicator}
  class:entry-row--rounded={rounded}
  class:entry-row--compact={compact}
  class={`entry-row ${className}`}
  aria-label={ariaLabel}
  aria-pressed={isSelected}
  {title}
  onclick={onSelect}
>
  <div class="entry-row__content">
    <div class="entry-row__main">
      {@render children?.()}
    </div>
    {#if meta}
      <div class="entry-row__meta">
        {@render meta()}
      </div>
    {/if}
  </div>
</button>

<style>
  .entry-row {
    --entry-row-indicator-color: var(--color-primary);
    --entry-row-separator-color: color-mix(
      in oklab,
      var(--color-base-content) 8%,
      transparent
    );
    --entry-row-depth-shadow: 0 0 0 0 transparent;
    --entry-row-selection-shadow: 0 0 0 0 transparent;

    display: flex;
    position: relative;
    width: 100%;
    max-width: 100%;
    min-width: 0;
    min-height: 4.5rem;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 3);
    border-left: 0;
    border-right: 0;
    border-top: 0;
    background-color: transparent;
    background-image:
      linear-gradient(
        var(--entry-row-separator-color),
        var(--entry-row-separator-color)
      ),
      linear-gradient(
        var(--entry-row-separator-color),
        var(--entry-row-separator-color)
      );
    background-position:
      top left,
      bottom left;
    background-repeat: no-repeat;
    background-size:
      100% var(--border),
      100% var(--border);
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    padding-inline: calc(var(--spacing) * 4) calc(var(--spacing) * 2);
    padding-block: calc(var(--spacing) * 2);
    box-shadow:
      var(--entry-row-depth-shadow), var(--entry-row-selection-shadow);
    transition:
      background-color 160ms ease,
      box-shadow 160ms ease;
  }

  .entry-row::before {
    position: absolute;
    inset-block: calc(var(--spacing) * 2);
    left: calc(var(--spacing) * 1);
    width: 0.375rem;
    border-radius: 999px;
    background-color: var(--entry-row-indicator-color);
    box-shadow: 0 0 0 1px
      color-mix(in oklab, var(--entry-row-indicator-color) 38%, transparent);
    content: '';
  }

  .entry-row--without-indicator {
    padding-inline-start: calc(var(--spacing) * 2);
  }

  .entry-row--without-indicator::before {
    display: none;
  }

  .entry-row--rounded {
    --entry-row-depth-shadow:
      0 0.5rem 1.25rem -1rem
        color-mix(in oklab, var(--color-base-content) 42%, transparent),
      0 1px 0 color-mix(in oklab, var(--color-base-content) 12%, transparent);

    width: calc(100% - calc(var(--spacing) * 2));
    margin: calc(var(--spacing) * 1);
    border-radius: var(--radius-box);
    background-color: color-mix(
      in oklab,
      var(--color-base-content) 4%,
      transparent
    );
    background-image: none;
  }

  .entry-row--compact {
    min-height: 3.75rem;
    gap: calc(var(--spacing) * 2);
    padding-block: calc(var(--spacing) * 1.5);
  }

  .entry-row--compact::before {
    inset-block: calc(var(--spacing) * 1.5);
  }

  .entry-row--matched {
    --entry-row-indicator-color: var(--color-success);
  }

  .entry-row--review {
    --entry-row-indicator-color: var(--color-warning);
  }

  .entry-row--unmatched {
    --entry-row-indicator-color: var(--color-error);
  }

  .entry-row--neutral {
    --entry-row-indicator-color: var(--color-primary);
  }

  .entry-row--info {
    --entry-row-indicator-color: var(--color-info);
  }

  .entry-row--suppressed {
    --entry-row-indicator-color: var(--ctp-mocha-maroon);
  }

  .entry-row--soft-warning {
    background-color: color-mix(in oklab, var(--color-error) 20%, transparent);
  }

  .entry-row--ignored {
    --entry-row-indicator-color: color-mix(
      in oklab,
      var(--color-base-content) 42%,
      transparent
    );

    background-color: color-mix(in oklab, black 16%, transparent);
    color: color-mix(in oklab, var(--color-base-content) 58%, transparent);
  }

  .entry-row__content {
    display: flex;
    width: 100%;
    min-width: 0;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 3);
  }

  .entry-row--compact .entry-row__content {
    gap: calc(var(--spacing) * 2);
  }

  .entry-row__main {
    min-width: 0;
  }

  .entry-row__meta {
    display: flex;
    flex: 0 0 auto;
    min-width: 3.75rem;
    flex-direction: column;
    align-items: flex-end;
    gap: calc(var(--spacing) * 1);
  }

  .entry-row:hover {
    background-color: color-mix(
      in oklab,
      var(--color-base-content) 5%,
      transparent
    );
  }

  .entry-row--soft-warning:hover {
    background-color: color-mix(in oklab, var(--color-error) 25%, transparent);
  }

  .entry-row--ignored:hover {
    background-color: color-mix(in oklab, black 20%, transparent);
  }

  .entry-row:focus-visible {
    outline: 1px solid
      color-mix(in oklab, var(--entry-row-indicator-color) 80%, white);
    outline-offset: -1px;
  }

  .entry-row--selected {
    --entry-row-selection-shadow: inset 0 0 0 2px
      color-mix(in oklab, var(--entry-row-indicator-color) 38%, transparent);

    background-color: color-mix(
      in oklab,
      var(--entry-row-indicator-color) 13%,
      transparent
    );
  }

  .entry-row--selected:hover {
    background-color: color-mix(
      in oklab,
      var(--entry-row-indicator-color) 17%,
      transparent
    );
  }
</style>
