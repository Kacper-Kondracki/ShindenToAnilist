<script>
  import Navbar from "./lib/Navbar.svelte";
  import Main from "./lib/Main.svelte";
  import Details from "./lib/Details.svelte";

  let currentView = "main";
  let anime = null;
  let username = ""
  let selected = "none"

  let showDetailView = false



  function switchToDetails(event) {
    currentView = "details";
    showDetailView = true
  }

  function switchTo(e) {
    if (e.detail === "main") {
      currentView = "main"
      showDetailView = false
    }
  }

</script>
<main class="flex h-screen flex-col">
  <Navbar on:goTo={switchTo} bind:showDetailView={showDetailView} />
  {#if currentView === "main"}
    <Main on:goToDetails={switchToDetails} bind:convertPromise={anime} bind:username bind:selected />
  {:else if currentView === "details"}
    <Details bind:anime />
  {/if}
</main>
