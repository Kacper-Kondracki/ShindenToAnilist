<script lang="ts">
  let {
    value = $bindable(),
    busy,
    canSubmit,
    hasError,
    errorMessage,
    onClearError,
    onSubmit,
  }: {
    value: string;
    busy: boolean;
    canSubmit: boolean;
    hasError: boolean;
    errorMessage?: string;
    onClearError: () => void;
    onSubmit: (event: SubmitEvent) => void;
  } = $props();
</script>

<form class="join flex-1" onsubmit={onSubmit}>
  <label
    class="user-list-input input join-item flex-1"
    class:input-error={hasError}
    title={errorMessage}
  >
    <span class="sr-only">ID lub nazwa użytkownika</span>
    <input
      bind:value
      type="text"
      placeholder="ID, profil Shinden lub nazwa użytkownika"
      autocomplete="off"
      oninput={onClearError}
      aria-invalid={hasError}
    />
  </label>
  <button
    class:load-button--active={busy}
    class="load-button btn join-item btn-info"
    type="submit"
    disabled={!canSubmit}
  >
    <span class="load-button__text">Wczytaj</span>
  </button>
</form>

<style>
  .user-list-input {
    transition: border-color 150ms ease;
  }

  .load-button {
    --load-button-highlight: color-mix(in oklab, white 44%, transparent);
    position: relative;
    isolation: isolate;
    overflow: hidden;
    min-width: 6rem;
    transition:
      transform 160ms ease,
      background-color 180ms ease,
      box-shadow 180ms ease,
      color 180ms ease;
  }

  .load-button::before,
  .load-button::after {
    position: absolute;
    border-radius: inherit;
    content: "";
    pointer-events: none;
  }

  .load-button::before {
    inset: -35% -70%;
    z-index: 0;
    background: linear-gradient(
      110deg,
      transparent 34%,
      var(--load-button-highlight) 46%,
      transparent 58%
    );
    opacity: 0;
    transform: translateX(-45%) rotate(5deg);
  }

  .load-button::after {
    inset: 1px;
    z-index: 0;
    background:
      radial-gradient(
        circle at 50% 20%,
        color-mix(in oklab, white 32%, transparent),
        transparent 75%
      ),
      linear-gradient(
        to bottom,
        color-mix(in oklab, white 14%, transparent),
        transparent 58%
      );
    opacity: 0;
    transition: opacity 180ms ease;
  }

  .load-button--active {
    filter: saturate(2) hue-rotate(0deg);
    --load-button-accent: color-mix(
      in oklab,
      var(--provider-accent) 75%,
      black
    );
    --btn-color: var(--load-button-accent);
    --load-button-highlight: color-mix(in oklab, white 62%, transparent);

    cursor: wait;
    border-color: color-mix(in oklab, var(--load-button-accent) 78%, white);
    background-color: var(--load-button-accent);
    color: var(--color-primary-content);
    box-shadow:
      0 0 0 1px color-mix(in oklab, var(--load-button-accent) 42%, transparent),
      0 0.45rem 1.1rem
        color-mix(in oklab, var(--load-button-accent) 34%, transparent),
      0 0 1.75rem color-mix(in oklab, var(--color-info) 26%, transparent);
    animation: load-button-halo 2.4s ease-in-out infinite;
  }

  .load-button--active::before {
    opacity: 0.9;
    animation: load-button-sheen 0.9s linear infinite;
  }

  .load-button--active::after {
    opacity: 0.72;
  }

  .load-button__text {
    position: relative;
    z-index: 1;
  }

  .load-button--active .load-button__text {
    animation: load-button-text 2.4s ease-in-out infinite;
  }

  @keyframes load-button-halo {
    0%,
    100% {
      background-color: color-mix(
        in oklab,
        var(--load-button-accent) 86%,
        var(--color-info)
      );
      box-shadow:
        0 0 0 1px
          color-mix(in oklab, var(--load-button-accent) 36%, transparent),
        0 0.4rem 1rem
          color-mix(in oklab, var(--load-button-accent) 28%, transparent),
        0 0 1.45rem color-mix(in oklab, var(--color-info) 20%, transparent);
    }

    50% {
      background-color: color-mix(
        in oklab,
        var(--load-button-accent) 78%,
        white
      );
      box-shadow:
        0 0 0 1px
          color-mix(in oklab, var(--load-button-accent) 56%, transparent),
        0 0.55rem 1.35rem
          color-mix(in oklab, var(--load-button-accent) 42%, transparent),
        0 0 1.95rem color-mix(in oklab, var(--color-info) 30%, transparent);
    }
  }

  @keyframes load-button-sheen {
    0% {
      transform: translateX(-45%) rotate(5deg);
    }

    58%,
    100% {
      transform: translateX(45%) rotate(5deg);
    }
  }

  @keyframes load-button-text {
    0%,
    100% {
      opacity: 0.92;
      text-shadow: 0 0 0 color-mix(in oklab, white 0%, transparent);
    }

    50% {
      opacity: 1;
      text-shadow: 0 0 0.75rem color-mix(in oklab, white 42%, transparent);
    }
  }
</style>
