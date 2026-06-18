<script lang="ts">
  import type { SourceImportProgress as SourceImportProgressState } from '../domain/anime';
  import { createSourceImportProgressController } from '../features/source/sourceImportProgressController.svelte';

  let {
    providerLabel,
    progress,
    onCancel
  }: {
    providerLabel: string;
    progress: SourceImportProgressState | null;
    onCancel: () => void;
  } = $props();

  const controller = createSourceImportProgressController({
    getProviderLabel: () => providerLabel,
    getProgress: () => progress
  });
</script>

<div
  class="source-progress"
  style:--source-progress-tone={controller.progressTone}
>
  <div class="source-progress__header">
    <div class="source-progress__title-group">
      <div class="source-progress__icon" aria-hidden="true">
        <span class="icon-[lucide--download-cloud]"></span>
      </div>
      <div class="source-progress__title-copy">
        <h2 title={controller.phaseText}>{controller.phaseText}</h2>
        <p
          class="source-progress__provider"
          title={`Importowanie listy z ${providerLabel}`}
        >
          Importowanie listy z {providerLabel}
        </p>
      </div>
    </div>

    <button
      class="btn btn-error btn-soft source-progress__cancel"
      type="button"
      aria-label={`Anuluj import z ${providerLabel}`}
      onclick={onCancel}
    >
      <span aria-hidden="true" class="icon-[lucide--circle-x] size-4"></span>
      <span class="source-progress__cancel-text">Anuluj</span>
    </button>
  </div>

  <div class="source-progress__bar-section">
    <div
      class="source-progress__track"
      role="progressbar"
      aria-label={`Postęp importu ${providerLabel}`}
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={controller.progressPercent}
      style:--source-progress-percent={`${controller.progressPercent}%`}
    >
      <span class="source-progress__fill"></span>
    </div>

    <div class="source-progress__bar-meta">
      <span>{controller.elapsedText}</span>
    </div>
  </div>

  <section
    class="source-progress__recent"
    class:source-progress__recent--waiting={!controller.hasRecentTitles}
    aria-label="Postęp importu"
  >
    {#if controller.hasRecentTitles}
      <div class="source-progress__recent-header">
        <span>{controller.progressPercent}%</span>
        <span>{controller.progressCountText}</span>
      </div>
      <div class="source-progress__recent-stream" role="log" aria-live="polite">
        {#each controller.recentTitleRows as row (row.id)}
          <div
            class="source-progress__recent-line"
            style:grid-row={row.gridRow}
            style:--source-entry-tone={row.tone}
            title={row.title}
          >
            <span aria-hidden="true" class="source-progress__recent-dot"></span>
            <span class="source-progress__recent-title">{row.title}</span>
          </div>
        {/each}
      </div>
    {:else}
      <div class="source-progress__recent-empty">
        <span class="source-progress__recent-waiting skeleton-text skeleton">
          Wczytywanie danych
        </span>
      </div>
    {/if}
  </section>
</div>

<style>
  .source-progress {
    --source-glass-border-width: calc(var(--border) * 2);
    --source-glass-border-color: color-mix(
      in oklab,
      var(--provider-accent, var(--color-primary)) 18%,
      var(--ctp-mocha-surface1) 82%
    );

    display: grid;
    position: relative;
    width: min(100%, 42rem);
    height: 24rem;
    grid-template-rows: auto auto 1fr;
    gap: calc(var(--spacing) * 5);
    border: var(--source-glass-border-width) solid
      var(--source-glass-border-color);
    border-radius: var(--radius-box);
    background:
      linear-gradient(
        180deg,
        color-mix(in oklab, white 5%, transparent),
        color-mix(in oklab, var(--ctp-mocha-surface0) 18%, transparent) 28%,
        color-mix(in oklab, var(--ctp-mocha-base) 22%, transparent)
      ),
      linear-gradient(
        180deg,
        color-mix(in oklab, var(--ctp-mocha-surface0) 84%, transparent),
        color-mix(in oklab, var(--ctp-mocha-base) 80%, transparent)
      ),
      linear-gradient(
        135deg,
        color-mix(
          in oklab,
          var(--provider-accent, var(--color-primary)) 5%,
          transparent
        ),
        color-mix(in oklab, var(--ctp-mocha-mantle) 24%, transparent) 56%,
        transparent
      );
    backdrop-filter: blur(1.65rem) saturate(1.22) brightness(0.94);
    -webkit-backdrop-filter: blur(1.65rem) saturate(1.22) brightness(0.94);
    padding: calc(var(--spacing) * 6);
    box-shadow:
      0 1.9rem 5.5rem -2.4rem color-mix(in oklab, black 92%, transparent),
      0 0 5.5rem -2.35rem
        color-mix(
          in oklab,
          var(--provider-accent, var(--color-primary)) 38%,
          transparent
        ),
      inset 0 1px 0 color-mix(in oklab, white 24%, transparent),
      inset 0 0 0 1px color-mix(in oklab, white 8%, transparent),
      inset 0 0 3rem
        color-mix(in oklab, var(--ctp-mocha-crust) 18%, transparent);
    overflow: hidden;
  }

  .source-progress::before,
  .source-progress::after {
    position: absolute;
    inset: 0;
    z-index: 0;
    content: '';
    pointer-events: none;
  }

  .source-progress::before {
    background:
      linear-gradient(
        120deg,
        color-mix(in oklab, white 8%, transparent),
        transparent 30%
      ),
      linear-gradient(
        305deg,
        color-mix(
            in oklab,
            var(--provider-accent, var(--color-primary)) 12%,
            transparent
          )
          0%,
        transparent 34%
      );
  }

  .source-progress::after {
    inset: var(--source-glass-border-width);
    border: var(--border) solid color-mix(in oklab, white 7%, transparent);
    border-radius: calc(var(--radius-box) - var(--source-glass-border-width));
    background: linear-gradient(
      180deg,
      color-mix(in oklab, white 7%, transparent),
      transparent 22%,
      color-mix(in oklab, var(--ctp-mocha-crust) 14%, transparent) 100%
    );
    opacity: 0.42;
    mix-blend-mode: screen;
  }

  .source-progress > * {
    position: relative;
    z-index: 1;
  }

  .source-progress__header {
    display: flex;
    min-width: 0;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 4);
    white-space: nowrap;
  }

  .source-progress__icon {
    display: grid;
    width: 3.25rem;
    height: 3.25rem;
    flex: 0 0 auto;
    place-items: center;
    border: calc(var(--border) * 2) solid
      color-mix(
        in oklab,
        var(--provider-accent, var(--color-primary)) 44%,
        white 14%
      );
    border-radius: var(--radius-box);
    background:
      linear-gradient(
        180deg,
        color-mix(in oklab, white 18%, transparent),
        transparent 62%
      ),
      color-mix(
        in oklab,
        var(--provider-accent, var(--color-primary)) 18%,
        transparent
      );
    color: var(--provider-accent, var(--color-primary));
    font-size: 1.5rem;
    box-shadow:
      inset 0 1px 0 color-mix(in oklab, white 32%, transparent),
      inset 0 0 0 1px color-mix(in oklab, white 10%, transparent),
      0 0.8rem 1.75rem -1.45rem
        color-mix(
          in oklab,
          var(--provider-accent, var(--color-primary)) 68%,
          transparent
        );
  }

  .source-progress__title-group {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: calc(var(--spacing) * 3);
  }

  .source-progress__title-copy {
    display: grid;
    min-width: 0;
    gap: calc(var(--spacing) * 1);
  }

  .source-progress__title-copy > h2,
  .source-progress__provider,
  .source-progress__bar-meta,
  .source-progress__recent-header,
  .source-progress__recent-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .source-progress__title-copy > h2 {
    margin: 0;
    font-size: clamp(1.25rem, 1.5vw, 1.65rem);
    font-weight: 800;
    line-height: 1.1;
  }

  .source-progress__provider {
    color: color-mix(in oklab, var(--color-base-content) 68%, transparent);
    font-size: 0.9rem;
    font-weight: 650;
    line-height: 1.1;
  }

  .source-progress__cancel {
    flex: 0 0 auto;
    border: calc(var(--border) * 2) solid
      color-mix(in oklab, var(--color-error) 36%, white 18%);
    background:
      linear-gradient(
        180deg,
        color-mix(in oklab, white 20%, transparent),
        transparent 62%
      ),
      color-mix(in oklab, var(--color-error) 16%, transparent);
    color: color-mix(in oklab, var(--color-error) 86%, white 14%);
    backdrop-filter: blur(1rem) saturate(1.28) brightness(1.04);
    -webkit-backdrop-filter: blur(1rem) saturate(1.28) brightness(1.04);
    box-shadow:
      inset 0 1px 0 color-mix(in oklab, white 30%, transparent),
      inset 0 0 0 1px color-mix(in oklab, white 10%, transparent),
      0 0.8rem 1.8rem -1.45rem
        color-mix(in oklab, var(--color-error) 62%, transparent);
  }

  .source-progress__cancel:hover {
    border-color: color-mix(in oklab, var(--color-error) 46%, white 20%);
    background:
      linear-gradient(
        180deg,
        color-mix(in oklab, white 24%, transparent),
        transparent 58%
      ),
      color-mix(in oklab, var(--color-error) 22%, transparent);
    box-shadow:
      inset 0 1px 0 color-mix(in oklab, white 36%, transparent),
      inset 0 0 0 1px color-mix(in oklab, white 12%, transparent),
      0 1rem 2rem -1.35rem
        color-mix(in oklab, var(--color-error) 72%, transparent);
  }

  .source-progress__cancel:active {
    box-shadow:
      inset 0 0.16rem 0.55rem color-mix(in oklab, black 22%, transparent),
      inset 0 1px 0 color-mix(in oklab, white 20%, transparent),
      0 0.6rem 1.35rem -1.2rem
        color-mix(in oklab, var(--color-error) 58%, transparent);
  }

  .source-progress__bar-section {
    display: grid;
    min-width: 0;
    gap: calc(var(--spacing) * 3);
  }

  .source-progress__track {
    position: relative;
    height: 1.4rem;
    border: calc(var(--border) * 2) solid
      color-mix(in oklab, white 18%, transparent);
    border-radius: 999px;
    background:
      linear-gradient(
        180deg,
        color-mix(in oklab, white 10%, transparent),
        transparent
      ),
      color-mix(in oklab, var(--color-base-300) 78%, transparent);
    box-shadow:
      inset 0 0.18rem 0.7rem color-mix(in oklab, black 28%, transparent),
      inset 0 1px 0 color-mix(in oklab, white 18%, transparent),
      0 0.65rem 1.4rem -1.15rem
        color-mix(in oklab, var(--source-progress-tone) 86%, transparent);
    overflow: hidden;
  }

  .source-progress__fill {
    position: absolute;
    inset-block: 0;
    inset-inline-start: 0;
    width: var(--source-progress-percent);
    min-width: 0.45rem;
    border-radius: inherit;
    background-color: var(--source-progress-tone);
    box-shadow:
      inset 0 1px 0 color-mix(in oklab, white 26%, transparent),
      0 0 1.1rem -0.4rem
        color-mix(in oklab, var(--source-progress-tone) 80%, transparent);
    transition: width 260ms ease;
  }

  .source-progress__bar-meta {
    display: flex;
    min-width: 0;
    justify-content: flex-end;
    color: color-mix(in oklab, var(--color-base-content) 68%, transparent);
    font-size: 0.86rem;
    font-weight: 750;
    line-height: 1;
    white-space: nowrap;
  }

  .source-progress__bar-meta > span {
    min-width: 4.5ch;
    font-variant-numeric: tabular-nums;
    text-align: end;
  }

  .source-progress__recent {
    display: grid;
    min-height: 0;
    grid-template-rows: auto minmax(0, 1fr);
    gap: calc(var(--spacing) * 3);
    border: var(--border) solid color-mix(in oklab, white 8%, transparent);
    border-radius: var(--radius-box);
    padding: calc(var(--spacing) * 3);
    box-shadow: inset 0 1px 0 color-mix(in oklab, white 8%, transparent);
  }

  .source-progress__recent--waiting {
    position: relative;
    grid-template-rows: minmax(0, 1fr);
  }

  .source-progress__recent-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 3);
    color: color-mix(in oklab, var(--color-base-content) 76%, transparent);
    font-size: 0.88rem;
    font-weight: 800;
    line-height: 1;
  }

  .source-progress__recent-header > span:first-child {
    color: var(--source-progress-tone);
    font-size: 1rem;
  }

  .source-progress__recent-empty {
    display: grid;
    position: relative;
    z-index: 1;
    min-width: 0;
    min-height: 0;
    place-items: center;
  }

  .source-progress__recent-waiting {
    display: inline-flex;
    max-width: 100%;
    min-height: 2.75rem;
    align-items: center;
    justify-content: center;
    border: var(--border) solid color-mix(in oklab, white 10%, transparent);
    border-radius: 999px;
    color: color-mix(in oklab, var(--color-base-content) 62%, transparent);
    font-size: 1.05rem;
    font-weight: 800;
    line-height: 1;
    overflow: hidden;
    padding-inline: calc(var(--spacing) * 6);
    box-shadow: inset 0 1px 0 color-mix(in oklab, white 8%, transparent);
    text-align: center;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .source-progress__recent-stream {
    display: grid;
    min-height: 0;
    grid-template-rows: repeat(5, minmax(0, 1fr));
    gap: calc(var(--spacing) * 1.5);
    align-content: end;
    overflow: hidden;
  }

  .source-progress__recent-line {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: calc(var(--spacing) * 2);
    border: var(--border) solid
      color-mix(in oklab, var(--source-entry-tone) 12%, transparent);
    border-radius: calc(var(--radius-field) * 0.75);
    background-color: color-mix(
      in oklab,
      var(--source-entry-tone) 11%,
      transparent
    );
    padding-inline: calc(var(--spacing) * 2);
    color: var(--source-entry-tone);
    font-size: 0.9rem;
    font-weight: 650;
    white-space: nowrap;
    overflow: hidden;
  }

  .source-progress__recent-dot {
    width: 0.42rem;
    height: 0.42rem;
    flex: 0 0 auto;
    border-radius: 999px;
    background-color: var(--source-entry-tone);
    opacity: 0.86;
  }

  @media (width <= 42rem) {
    .source-progress {
      height: 23rem;
      gap: calc(var(--spacing) * 4);
      padding: calc(var(--spacing) * 4);
    }

    .source-progress__icon {
      width: 2.75rem;
      height: 2.75rem;
      font-size: 1.3rem;
    }

    .source-progress__title-copy > h2 {
      font-size: 1.12rem;
    }

    .source-progress__cancel {
      width: 2.35rem;
      padding-inline: 0;
    }

    .source-progress__cancel-text {
      display: none;
    }

    .source-progress__provider {
      font-size: 0.82rem;
    }
  }
</style>
