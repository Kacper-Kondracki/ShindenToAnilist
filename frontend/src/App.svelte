<script lang="ts">
  let counter = $state(0);
</script>

<div class="min-h-dvh flex flex-col gap-4 justify-center items-center">
  <h1>Count: {counter}</h1>
  <button onclick={() => counter++} class="btn btn-primary">Increment</button>
</div>

<style>
  /* Put your standard CSS here */
</style>
