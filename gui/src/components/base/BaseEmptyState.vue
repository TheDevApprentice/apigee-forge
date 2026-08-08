<template>
  <div class="empty-state" role="status">
    <div class="empty-state__mark" aria-hidden="true">—</div>
    <p class="empty-state__title"><slot name="title">Nothing here yet</slot></p>
    <p class="empty-state__hint"><slot name="hint">Connect a workspace to see live data.</slot></p>
  </div>
</template>
