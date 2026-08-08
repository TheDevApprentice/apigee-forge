<template>
  <div class="error-state" role="alert">
    <p class="error-state__title"><slot name="title">Something went wrong</slot></p>
    <p class="error-state__hint"><slot name="hint">Please try again.</slot></p>
    <button class="error-state__action" type="button">
      <slot name="action">Retry</slot>
    </button>
  </div>
</template>
