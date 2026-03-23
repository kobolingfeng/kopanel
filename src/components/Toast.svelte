<script lang="ts">
  import { getToasts, dismissToast } from '../lib/toast.svelte';

  // 获取响应式 toasts 列表
  let toasts = $derived(getToasts());
</script>

{#if toasts.length > 0}
  <div class="toast-container">
    {#each toasts as toast (toast.id)}
      <div
        class="toast toast-{toast.type}"
        role="button"
        tabindex="0"
        onclick={() => dismissToast(toast.id)}
        onkeydown={(e) => e.key === 'Enter' && dismissToast(toast.id)}
      >
        <div class="toast-icon">
          {#if toast.type === 'success'}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"><path d="M20 6L9 17l-5-5" /></svg
            >
          {:else if toast.type === 'error'}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              ><circle cx="12" cy="12" r="10" /><line
                x1="15"
                y1="9"
                x2="9"
                y2="15"
              /><line x1="9" y1="9" x2="15" y2="15" /></svg
            >
          {:else if toast.type === 'warning'}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              ><path
                d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
              /><line x1="12" y1="9" x2="12" y2="13" /><line
                x1="12"
                y1="17"
                x2="12.01"
                y2="17"
              /></svg
            >
          {:else}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              ><circle cx="12" cy="12" r="10" /><line
                x1="12"
                y1="16"
                x2="12"
                y2="12"
              /><line x1="12" y1="8" x2="12.01" y2="8" /></svg
            >
          {/if}
        </div>
        <span class="toast-message">{toast.message}</span>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 10000;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border-radius: 8px;
    font-size: 13px;
    color: white;
    background: rgba(0, 0, 0, 0.85);
    backdrop-filter: blur(10px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    animation: toast-slide-in 0.3s ease-out;
    pointer-events: auto;
    cursor: pointer;
    transition: transform 0.15s ease, opacity 0.15s ease;
  }

  .toast:hover {
    transform: scale(1.02);
  }

  .toast:active {
    transform: scale(0.98);
  }

  .toast-success {
    background: linear-gradient(135deg, rgba(34, 197, 94, 0.9), rgba(22, 163, 74, 0.9));
  }

  .toast-error {
    background: linear-gradient(135deg, rgba(239, 68, 68, 0.9), rgba(220, 38, 38, 0.9));
  }

  .toast-warning {
    background: linear-gradient(135deg, rgba(245, 158, 11, 0.9), rgba(217, 119, 6, 0.9));
  }

  .toast-info {
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.9), rgba(37, 99, 235, 0.9));
  }

  .toast-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .toast-message {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 300px;
  }

  @keyframes toast-slide-in {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
