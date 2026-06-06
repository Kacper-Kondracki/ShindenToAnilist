<script lang="ts">
  import type { Provider, ProviderOption } from "../config/providers";

  let {
    providers,
    selectedProvider = $bindable(),
  }: {
    providers: readonly ProviderOption[];
    selectedProvider: Provider;
  } = $props();
</script>

<div class="join shrink-0">
  {#each providers as provider}
    <button
      type="button"
      class:provider-button--selected={selectedProvider === provider.id}
      class:btn-ghost={selectedProvider !== provider.id}
      class="provider-button btn join-item border-0 btn-soft"
      style:--provider-button-accent={provider.accent}
      disabled={provider.disabled}
      aria-pressed={selectedProvider === provider.id}
      title={provider.site}
      onclick={() => (selectedProvider = provider.id as Provider)}
    >
      {provider.label}
    </button>
  {/each}
</div>

<style>
  .provider-button {
    --provider-button-color: var(
      --provider-button-accent,
      var(--color-primary)
    );
    --btn-color: var(--provider-button-color);
    --btn-fg: var(--provider-button-color);
    --btn-bg: color-mix(
      in oklab,
      var(--provider-button-color) 10%,
      var(--color-base-100)
    );
    --btn-border: transparent;
    --btn-noise: none;
    --btn-shadow: none;

    box-shadow: none;
    text-shadow: none;
  }

  .provider-button:not(.provider-button--selected) {
    color: var(--provider-button-color);
  }

  .provider-button:active:not(.btn-active) {
    --btn-border: transparent;
    --btn-shadow: none;

    box-shadow: none;
  }

  @media (hover: hover) {
    .provider-button:not(.provider-button--selected):hover {
      --btn-fg: var(--provider-button-color);
      --btn-bg: color-mix(
        in oklab,
        var(--provider-button-color) 25%,
        var(--color-base-100)
      );
      --btn-border: transparent;
      --btn-shadow: none;
    }

    .provider-button--selected:hover {
      --btn-bg: color-mix(in oklab, var(--provider-button-color) 94%, white);
      --btn-border: var(--provider-button-color);
      --btn-shadow: none;
    }
  }

  .provider-button--selected {
    --btn-bg: var(--provider-button-color);
    --btn-fg: var(--color-primary-content);
    border-color: var(--provider-button-color);
    background-color: var(--provider-button-color);
    color: var(--color-primary-content);
  }
</style>
