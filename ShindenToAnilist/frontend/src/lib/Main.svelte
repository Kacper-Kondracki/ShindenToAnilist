
  <script>
    import QuestionIcon from "virtual:icons/mdi/InfoOutline";
    import ErrorIcon from "virtual:icons/mdi/WarningOutline";
    import SuccessIcon from "virtual:icons/mdi/SuccessCircleOutline";

    import shindenImg from "../assets/shinden.jpg";
    import animezoneImg from "../assets/animezone.jpg";
    import ogladajanimeImg from "../assets/ogladajanime.jpg";

    import { Convert } from "../../wailsjs/go/main/App.js";
    import SiteButton from "../lib/SiteButton.svelte";
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();
    function goToDetails() {
      dispatch('goToDetails', convertPromise);
    }

    const Sites = Object.freeze({
    SHINDEN: "shinden",
    ANIMEZONE: "animezone",
    OGLADAJANIME: "ogladajanime",
    NONE: "none",
  });

    export let selected = "none";
    export let username = "";
    export let convertPromise = null;

    function convert() {
      convertPromise = Convert(username, selected);
  }
</script>

<div
  style="max-height: 1000px;"
  class="m-auto flex flex-grow flex-col items-stretch justify-between gap-5 p-5"
>
  <!-- Input box -->
  <div class="join">
    <input
      type="text"
      bind:value={username}
      placeholder="Podaj dane"
      disabled={selected === Sites.NONE}
      class="input join-item input-bordered input-accent w-full focus:outline-none {selected ===
        Sites.NONE
          ? 'input-disabled'
          : ''}"
    />
    <button
      disabled={selected === Sites.NONE}
      on:click={convert}
      class="btn btn-accent join-item {selected === Sites.NONE
          ? 'button-disabled'
          : ''}"
    >Wyszukaj
    </button>
  </div>
  <!-- Content/loading circle -->
  <div class="self-center">
    {#if convertPromise != null}
      {#await convertPromise}
        <span class="loading loading-ring w-36"></span>
      {:then result}
        {#if result.status === "OK"}
          {#if result.successCount + result.multipleCount + result.failCount > 0}
            <div class="flex flex-col">
              <div class="stats shadow">
                <div class="stat">
                  <div class="stat-figure text-info">
                    <QuestionIcon class="-m-2 size-10"></QuestionIcon>
                  </div>

                  <div class="stat-title">Ilość serii</div>
                  <div class="stat-value">
                    {result.successCount + result.multipleCount + result.failCount}
                  </div>
                  <div class="stat-desc"><wbr /></div>
                </div>

                <div class="stat">
                  <div class="stat-figure text-success">
                    <SuccessIcon class="-m-2 size-10"></SuccessIcon>
                  </div>

                  <div class="stat-title">Znaleziono</div>
                  <div class="stat-value">{result.successCount !== 0 ? result.successCount : '-'}</div>
                  <div class="stat-desc">
                    {Math.floor(
                      (result.successCount /
                        (result.successCount + result.multipleCount + result.failCount)) *
                      100,
                    )}% serii
                  </div>
                </div>

                <div class="stat">
                  <div class="stat-figure text-error">
                    <ErrorIcon class="-m-2 size-10"></ErrorIcon>
                  </div>

                  <div class="stat-title">Nie znaleziono</div>
                  <div class="stat-value">
                    {result.multipleCount + result.failCount !== 0 ? result.multipleCount + result.failCount : '-' }
                  </div>
                  <div class="stat-desc">
                    {#if result.multipleCount > 0}
                      w tym {result.multipleCount} niepewnych
                    {:else}
                      <wbr />
                    {/if}
                  </div>
                </div>
              </div>
              <button class="btn btn-accent" on:click={goToDetails}>Przejdź dalej</button>
            </div>
          {:else}
            <span>Brak anime</span>
          {/if}
        {:else}
          <span>Wystąpił błąd.</span>
        {/if}
      {/await}
    {/if}
  </div>

  <!-- Service selection -->
  <div class="flex flex-col gap-5">
    <!-- Alert -->
    <div role="alert" class="alert">
      <QuestionIcon class="h-6 w-6 shrink-0 text-info" />
      {#if selected === Sites.SHINDEN}
        <span>Podaj ID lub link profilu użytkownika Shindena.</span>
      {:else if selected === Sites.ANIMEZONE}
        <span>W budowie.</span>
      {:else if selected === Sites.OGLADAJANIME}
        <span>W budowie.</span>
      {:else}
        <span>Wybierz odpowiedni serwis.</span>
      {/if}
    </div>
    <!-- Site Buttons -->
    <div class="fajn flex justify-between gap-5">
      <SiteButton
        image={shindenImg}
        text="Shinden"
        selected={selected === Sites.SHINDEN}
        on:click={() => (selected = Sites.SHINDEN)}
      ></SiteButton>
      <SiteButton
        image={animezoneImg}
        text="Anime Zone"
        selected={selected === Sites.ANIMEZONE}
        on:click={() => (selected = Sites.ANIMEZONE)}
      ></SiteButton>
      <SiteButton
        image={ogladajanimeImg}
        text="Oglądaj Anime"
        selected={selected === Sites.OGLADAJANIME}
        on:click={() => (selected = Sites.OGLADAJANIME)}
      ></SiteButton>
    </div>
  </div>
</div>