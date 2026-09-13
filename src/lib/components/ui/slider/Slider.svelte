<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';
  
  interface Props extends Omit<HTMLInputAttributes, 'type'> {
    value?: number;
  }
  
  let { value = $bindable(128), class: className, ...restProps }: Props = $props();
</script>

<div class="relative flex w-full touch-none select-none items-center {className}">
  <input
    type="range"
    bind:value
    class="w-full h-2 bg-border rounded-full appearance-none cursor-pointer accent-primary focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2"
    {...restProps}
  />
</div>

<style>
  /* Custom cross-browser range slider styling for aesthetics */
  input[type=range]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 1.25rem;
    height: 1.25rem;
    border-radius: 50%;
    background: #a78bfa;
    cursor: pointer;
    box-shadow: 0 1px 3px rgba(0,0,0,0.3);
    transition: transform 0.1s;
  }
  input[type=range]::-webkit-slider-thumb:hover {
    transform: scale(1.1);
  }
  input[type=range]::-moz-range-thumb {
    width: 1.25rem;
    height: 1.25rem;
    border-radius: 50%;
    background: #a78bfa;
    cursor: pointer;
    box-shadow: 0 1px 3px rgba(0,0,0,0.3);
    border: none;
    transition: transform 0.1s;
  }
  input[type=range]::-moz-range-thumb:hover {
    transform: scale(1.1);
  }
</style>
