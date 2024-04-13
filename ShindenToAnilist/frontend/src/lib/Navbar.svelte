<script>
import { BrowserOpenURL } from "../../wailsjs/runtime/runtime.js";
import GithubIcon from "virtual:icons/mdi/Github";
import { createEventDispatcher } from "svelte";

const dispatch = createEventDispatcher();
function goTo(e) {
  dispatch('goTo', e);
}
export let showDetailView = false
</script>  <!-- Navbar -->
<div class="navbar bg-base-200">
  <div class="navbar-start">
    <button on:click={() => goTo("main")} class="btn btn-ghost text-xl">Shinden to Anilist</button>
  </div>
  {#if showDetailView}
  <div class="navbar-center">
    <button on:click={() => goTo("details")} class="btn btn-accent text-xl">Dopasuj nieznalezione</button>
  </div>
    {/if}
  <div class="navbar-end">
    <button
      on:click={() =>
          BrowserOpenURL(
            "https://github.com/Kacper-Kondracki/ShindenToAnilist",
          )}
      class="btn"
    >
      <GithubIcon style="font-size: 2em;" />
    </button>
  </div>
</div>