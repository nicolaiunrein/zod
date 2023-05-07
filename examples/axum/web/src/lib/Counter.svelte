<script lang="ts">
  import { onMount } from "svelte";
  import { connect } from "../client";
  import { Watchout, Pixera } from "../api";

  let client = connect("ws://localhost:8000/ws");

  let watchout = Watchout.init(client);
  let pixera = Pixera.init(client);

  let count: BigInt = 0n

  const increment = async () => {
    count = await watchout.hello("abc", 123n);
  }

  const x = watchout.hello_stream(20n);
  const y = pixera.hello_stream(20n);
  const s = pixera.debug_stream();

  s.subscribe(xx => console.log(xx))

  onMount(async () => {
    count = await watchout.hello("abc", 123n);
  });

</script>

<button on:click={increment}>
  count is {count}
</button>
<p>Watchout: {$x}</p>
<p>Pixera: {$y}</p>
<p>S: {$s}</p>

