<script lang="ts">
  import { onMount } from "svelte";
  import { AppService } from "../bindings/shindentoanilist";

  let count = $state<number>(0);
  let error = $state("");

  onMount(() => {
    void loadCounter();
  });

  async function loadCounter() {
    error = "";
    try {
      count = await AppService.CounterValue();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    }
  }

  async function incrementCounter() {
    error = "";
    try {
      count = await AppService.IncrementCounter();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    }
  }
</script>

<div class="min-h-dvh flex flex-col gap-4 justify-center items-center">
  <h1>Count: {count}</h1>
  <button onclick={incrementCounter} class="btn btn-primary">
    Increment
  </button>
  {#if error}
    <p class="text-error">{error}</p>
  {/if}
</div>

<style>
  /* Put your standard CSS here */
</style>
