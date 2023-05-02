<script lang="ts">
 import {Watchout, Pixera} from "../api";

  let count: number = 0
  const increment = () => {
    count += 1
  }

  const x = Watchout.hello_stream(20n);
  const y = Pixera.hello_stream(20n);

</script>

<button on:click={increment}>
  count is {count}
  {$x}
  {$y}
</button>
