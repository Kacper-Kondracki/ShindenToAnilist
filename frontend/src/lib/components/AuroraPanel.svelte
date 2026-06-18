<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    children,
    class: className = ''
  }: { children?: Snippet; class?: string } = $props();
</script>

<section class={`aurora-panel ${className}`}>
  {@render children?.()}
</section>

<style>
  .aurora-panel {
    display: grid;
    position: relative;
    isolation: isolate;
    min-height: 0;
    place-items: center;
    border: calc(var(--border) * 2) solid
      color-mix(
        in oklab,
        var(--provider-accent, var(--ctp-mocha-sky)) 34%,
        white 8%
      );
    border-radius: var(--radius-box);
    background:
      linear-gradient(
        180deg,
        color-mix(
          in oklab,
          var(--provider-accent, var(--ctp-mocha-sky)) 12%,
          transparent
        ),
        transparent 44%
      ),
      linear-gradient(
        135deg,
        color-mix(in oklab, var(--ctp-mocha-crust) 92%, black),
        color-mix(in oklab, var(--ctp-mocha-mantle) 88%, black) 45%,
        color-mix(in oklab, var(--ctp-mocha-base) 92%, black)
      );
    box-shadow:
      inset 0 1px 0 color-mix(in oklab, white 24%, transparent),
      inset 0 0 0 1px color-mix(in oklab, white 10%, transparent),
      inset 0 0 2.75rem color-mix(in oklab, white 5%, transparent),
      0 1.25rem 4rem -2.35rem
        color-mix(
          in oklab,
          var(--provider-accent, var(--ctp-mocha-sky)) 52%,
          transparent
        );
    overflow: hidden;
  }

  .aurora-panel::before,
  .aurora-panel::after {
    position: absolute;
    inset: 0;
    z-index: 0;
    content: '';
    pointer-events: none;
  }

  .aurora-panel::before {
    inset: -18%;
    background:
      linear-gradient(
        118deg,
        transparent 9%,
        color-mix(
            in oklab,
            var(--provider-accent, var(--ctp-mocha-sky)) 28%,
            transparent
          )
          23%,
        transparent 39%
      ),
      linear-gradient(
        63deg,
        transparent 28%,
        color-mix(in oklab, var(--ctp-mocha-teal) 18%, transparent) 47%,
        transparent 62%
      ),
      linear-gradient(
        151deg,
        transparent 48%,
        color-mix(in oklab, var(--ctp-mocha-lavender) 20%, transparent) 64%,
        transparent 78%
      ),
      linear-gradient(
        180deg,
        transparent,
        color-mix(in oklab, var(--ctp-mocha-crust) 72%, transparent) 72%
      );
    background-position:
      0% 50%,
      100% 42%,
      42% 100%,
      50% 50%;
    background-size:
      140% 140%,
      150% 150%,
      135% 135%,
      100% 100%;
    filter: saturate(1.45) blur(0.15rem);
    opacity: 0.98;
    transform: rotate(-4deg) scale(1.04);
    will-change: transform, background-position, filter;
  }

  .aurora-panel::after {
    background:
      linear-gradient(
        112deg,
        transparent 8%,
        color-mix(in oklab, var(--ctp-mocha-rosewater) 12%, transparent) 34%,
        transparent 58%
      ),
      linear-gradient(
        244deg,
        transparent 14%,
        color-mix(in oklab, var(--ctp-mocha-sapphire) 14%, transparent) 46%,
        transparent 68%
      ),
      linear-gradient(
        180deg,
        color-mix(in oklab, white 5%, transparent),
        transparent 24%,
        color-mix(in oklab, var(--ctp-mocha-crust) 28%, transparent)
      );
    mask-image: linear-gradient(
      180deg,
      transparent,
      black 18%,
      black 84%,
      transparent
    );
    -webkit-mask-image: linear-gradient(
      180deg,
      transparent,
      black 18%,
      black 84%,
      transparent
    );
    opacity: 0.72;
    transform: translate3d(0, 0, 0);
    will-change: transform, background-position, opacity, filter;
  }

  .aurora-panel > :global(*) {
    position: relative;
    z-index: 1;
  }

  @media (prefers-reduced-motion: no-preference) {
    .aurora-panel::before {
      animation: aurora-panel-flow 24s cubic-bezier(0.33, 0.18, 0.67, 0.82)
        infinite;
    }

    .aurora-panel::after {
      animation: aurora-panel-glow 34s cubic-bezier(0.33, 0.18, 0.67, 0.82)
        infinite;
    }
  }

  @keyframes aurora-panel-flow {
    0% {
      background-position:
        0% 50%,
        100% 42%,
        42% 100%,
        50% 50%;
      filter: saturate(1.35) blur(0.18rem);
      transform: translate3d(-2.4rem, -1rem, 0) rotate(-5deg) scale(1.04);
    }

    24% {
      background-position:
        42% 36%,
        66% 70%,
        76% 30%,
        50% 50%;
      filter: saturate(1.52) blur(0.1rem);
      transform: translate3d(0.4rem, 1.3rem, 0) rotate(-3deg) scale(1.08);
    }

    52% {
      background-position:
        86% 44%,
        30% 58%,
        58% 18%,
        50% 50%;
      filter: saturate(1.68) blur(0.04rem);
      transform: translate3d(1.8rem, 1rem, 0) rotate(-0.5deg) scale(1.11);
    }

    78% {
      background-position:
        76% 64%,
        18% 30%,
        28% 18%,
        50% 50%;
      filter: saturate(1.46) blur(0.14rem);
      transform: translate3d(1.4rem, -1rem, 0) rotate(1.5deg) scale(1.09);
    }

    100% {
      background-position:
        0% 50%,
        100% 42%,
        42% 100%,
        50% 50%;
      filter: saturate(1.35) blur(0.18rem);
      transform: translate3d(-2.4rem, -1rem, 0) rotate(-5deg) scale(1.04);
    }
  }

  @keyframes aurora-panel-glow {
    0% {
      background-position:
        18% 42%,
        82% 54%,
        50% 50%;
      filter: saturate(1.1);
      opacity: 0.56;
      transform: translate3d(-1rem, -0.4rem, 0) scale(1);
    }

    28% {
      background-position:
        62% 34%,
        40% 64%,
        50% 50%;
      filter: saturate(1.24);
      opacity: 0.74;
      transform: translate3d(0.7rem, 0.8rem, 0) scale(1.02);
    }

    56% {
      background-position:
        78% 58%,
        26% 34%,
        50% 50%;
      filter: saturate(1.36);
      opacity: 0.84;
      transform: translate3d(1.4rem, 0.2rem, 0) scale(1.035);
    }

    82% {
      background-position:
        44% 66%,
        92% 24%,
        50% 50%;
      filter: saturate(1.2);
      opacity: 0.68;
      transform: translate3d(0.2rem, -1rem, 0) scale(1.015);
    }

    100% {
      background-position:
        18% 42%,
        82% 54%,
        50% 50%;
      filter: saturate(1.1);
      opacity: 0.56;
      transform: translate3d(-1rem, -0.4rem, 0) scale(1);
    }
  }
</style>
