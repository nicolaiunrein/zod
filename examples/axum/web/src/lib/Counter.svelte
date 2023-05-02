<script lang="ts">
 import {Watchout, Pixera} from "../api";
 import {onMount} from "svelte";

  let count: number = 0
  const increment = async () => {
    count = await Watchout.hello("abc", 123n);
  }

  const x = Watchout.hello_stream(20n);
  const y = Pixera.hello_stream(20n);

  onMount(async () => {
    count = await Watchout.hello("abc", 123n);
  });

</script>

<button on:click={increment}>
  count is {count}
</button>
<p>Watchout: {$x}</p>
<p>Pixera: {$y}</p>

