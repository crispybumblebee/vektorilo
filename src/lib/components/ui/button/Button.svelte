<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';
  
  interface Props extends HTMLButtonAttributes {
    variant?: 'default' | 'outline' | 'ghost';
    size?: 'default' | 'sm' | 'lg' | 'icon';
    children?: import('svelte').Snippet;
  }
  
  let { variant = 'default', size = 'default', class: className, children, ...restProps }: Props = $props();

  const variants = {
    default: 'bg-primary text-white hover:bg-primary-dark shadow-sm',
    outline: 'border border-primary text-primary hover:bg-primary-light',
    ghost: 'hover:bg-primary-light text-foreground',
  };
  
  const sizes = {
    default: 'h-10 px-6 py-2 rounded-full',
    sm: 'h-9 px-4 rounded-full',
    lg: 'h-12 px-8 rounded-full text-lg',
    icon: 'h-10 w-10 rounded-full',
  };
</script>

<button
  class="inline-flex items-center justify-center font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:pointer-events-none disabled:opacity-50 {variants[variant]} {sizes[size]} {className}"
  {...restProps}
>
  {@render children?.()}
</button>
