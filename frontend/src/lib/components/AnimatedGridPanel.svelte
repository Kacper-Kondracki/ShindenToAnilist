<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    children,
    class: className = "",
  }: { children?: Snippet; class?: string } = $props();
</script>

<section class={`animated-grid-panel ${className}`}>
  <div class="animated-grid-panel__grid" aria-hidden="true"></div>
  {@render children?.()}
</section>

<style>
  @property --animated-grid-accent {
    syntax: "<color>";
    inherits: true;
    initial-value: transparent;
  }
  @property --animated-grid-line {
    syntax: "<color>";
    inherits: true;
    initial-value: transparent;
  }
  @property --animated-grid-glow {
    syntax: "<color>";
    inherits: true;
    initial-value: transparent;
  }

  .animated-grid-panel {
    --animated-grid-accent: var(--provider-accent, var(--color-primary));
    --animated-grid-glow: color-mix(
      in oklab,
      var(--animated-grid-accent) 36%,
      transparent
    );
    --animated-grid-line: color-mix(
      in oklab,
      var(--animated-grid-accent) 40%,
      transparent
    );

    transition:
      --animated-grid-accent 100ms ease,
      --animated-grid-line 100ms ease,
      --animated-grid-glow 100ms ease,
      box-shadow 100ms ease;

    position: relative;
    box-shadow: inset 0 0 2rem var(--animated-grid-glow);
  }

  .animated-grid-panel::before,
  .animated-grid-panel::after,
  .animated-grid-panel__grid {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .animated-grid-panel::before,
  .animated-grid-panel::after {
    z-index: 1;
    content: "";
  }

  .animated-grid-panel::before {
    background: linear-gradient(
      180deg,
      var(--color-base-300),
      color-mix(in oklab, var(--color-base-300) 70%, transparent) 22%,
      transparent 56%
    );
  }

  .animated-grid-panel::after {
    background: radial-gradient(
      ellipse at center,
      transparent 44%,
      color-mix(in oklab, var(--color-base-300) 84%, transparent)
    );
  }

  .animated-grid-panel__grid {
    background-image:
      linear-gradient(var(--animated-grid-line) 2px, transparent 2px),
      linear-gradient(90deg, var(--animated-grid-line) 2px, transparent 2px);
    background-size: 4rem 4rem;
    animation: move-grid 6s linear infinite;
    filter: blur(1px);
  }

  @keyframes move-grid {
    to {
      background-position: 8rem 12rem;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .animated-grid-panel__grid {
      animation: none;
    }
  }
</style>
