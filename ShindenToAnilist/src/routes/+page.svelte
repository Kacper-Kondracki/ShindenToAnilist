<script lang="ts">
	import './layout.css';

	let count = $state(0);
</script>

<div class=" flex h-screen flex-col items-center justify-center gap-4">
	<h1>Hello World!</h1>
	<button class="btn btn-primary" onclick={() => count++}>Button</button>
	<h1>Counter: {count}</h1>
</div>
