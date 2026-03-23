<script lang="ts">
  import { listen } from '@tauri-apps/api/event';

  interface Props {
    /** 是否显示对话框 */
    show: boolean;
    /** 翻译函数 */
    t: (key: string) => string;
    /** 关闭对话框回调 */
    onClose: () => void;
    /** 导入成功回调 */
    onImportSuccess?: () => void;
  }

  let {
    show,
    t,
    onClose,
    onImportSuccess,
  }: Props = $props();

  let installing = $state(false);
  let importing = $state(false);
  let downloading = $state(false);
  let downloadPercent = $state(0);
  let hasBundled = $state(false);
  let errorMsg = $state('');

  // 检查是否存在捆绑安装程序
  $effect(() => {
    if (show) {
      // 重新打开时清除上次残留的错误信息
      errorMsg = '';
      import('../lib/api')
        .then(({ rtssHasBundledInstaller }) => rtssHasBundledInstaller())
        .then((v) => {
          hasBundled = v;
        })
        .catch(() => {});
    }
  });

  const isBusy = $derived(installing || importing || downloading);

  async function handleDownload() {
    downloading = true;
    downloadPercent = 0;
    errorMsg = '';

    // 监听下载进度
    const unlisten = await listen<{ percent: number }>('rtss-download-progress', (event) => {
      downloadPercent = Math.round(event.payload.percent);
    });

    try {
      const { rtssDownloadInstaller } = await import('../lib/api');
      const result = await rtssDownloadInstaller();
      if (!result.success) {
        errorMsg = result.message || t('rtss_dialog_error');
      } else {
        onClose();
      }
    } catch (e: any) {
      errorMsg = e?.toString() || t('rtss_dialog_error');
    } finally {
      unlisten();
      downloading = false;
      downloadPercent = 0;
    }
  }

  async function handleInstall() {
    installing = true;
    errorMsg = '';
    try {
      const { rtssRunInstaller } = await import('../lib/api');
      const result = await rtssRunInstaller();
      if (!result.success) {
        errorMsg = result.message || t('rtss_dialog_error');
      } else {
        onClose();
      }
    } catch (e: any) {
      errorMsg = e?.toString() || t('rtss_dialog_error');
    } finally {
      installing = false;
    }
  }

  async function handleImport() {
    importing = true;
    errorMsg = '';
    try {
      const [{ open }, { rtssImportPath }] = await Promise.all([
        import('@tauri-apps/plugin-dialog'),
        import('../lib/api'),
      ]);
      const selected = await open({
        multiple: false,
        filters: [{ name: t('rtss_dialog_import_filter'), extensions: ['exe'] }],
      });
      if (!selected) {
        importing = false;
        return;
      }
      const result = await rtssImportPath(selected as string);
      if (!result.success) {
        errorMsg = result.message || t('rtss_dialog_error');
      } else {
        onImportSuccess?.();
        onClose();
      }
    } catch (e: any) {
      errorMsg = e?.toString() || t('rtss_dialog_error');
    } finally {
      importing = false;
    }
  }

  function handleSkip() {
    onClose();
  }
</script>

{#if show}
  <div class="rtss-dialog-overlay" role="dialog" aria-modal="true" aria-label={t('rtss_dialog_title')}>
    <div class="rtss-dialog">
      <div class="rtss-dialog-title">{t('rtss_dialog_title')}</div>
      <div class="rtss-dialog-desc">{t('rtss_dialog_desc')}</div>

      {#if errorMsg}
        <div class="rtss-dialog-error">{errorMsg}</div>
      {/if}

      <div class="rtss-dialog-actions">
        <!-- 下载安装（主按钮） -->
        <button
          class="rtss-btn rtss-btn-primary"
          onclick={handleDownload}
          disabled={isBusy}
        >
          {#if downloading}
            {t('rtss_dialog_downloading')} {downloadPercent}%
          {:else}
            {t('rtss_dialog_download')}
          {/if}
        </button>

        {#if downloading}
          <div class="rtss-progress-bar">
            <div class="rtss-progress-fill" style="width: {downloadPercent}%"></div>
          </div>
        {/if}

        <!-- 本地安装程序（仅当捆绑安装程序存在时显示） -->
        {#if hasBundled}
          <button
            class="rtss-btn rtss-btn-secondary"
            onclick={handleInstall}
            disabled={isBusy}
          >
            {installing ? t('rtss_dialog_installing') : t('rtss_dialog_install')}
          </button>
        {/if}

        <button
          class="rtss-btn rtss-btn-secondary"
          onclick={handleImport}
          disabled={isBusy}
        >
          {importing ? t('rtss_dialog_importing') : t('rtss_dialog_import')}
        </button>

        <button
          class="rtss-btn rtss-btn-ghost"
          onclick={handleSkip}
          disabled={isBusy}
        >
          {t('rtss_dialog_skip')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .rtss-dialog-overlay {
    position: fixed;
    inset: 0;
    z-index: 9000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
  }

  .rtss-dialog {
    background: var(--bg-gradient-start, #1b2838);
    border: 1px solid var(--border-color, rgba(61, 68, 80, 0.3));
    border-radius: var(--radius-lg, 12px);
    padding: 24px;
    max-width: 380px;
    width: 90%;
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }

  .rtss-dialog-title {
    font-size: calc(16px * var(--font-scale, 1));
    font-weight: 700;
    color: var(--text-primary, #fff);
  }

  .rtss-dialog-desc {
    font-size: calc(12px * var(--font-scale, 1));
    color: var(--text-secondary, #9ca3af);
    line-height: 1.5;
  }

  .rtss-dialog-error {
    font-size: calc(11px * var(--font-scale, 1));
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
    padding: 8px 12px;
    border-radius: 6px;
    border: 1px solid rgba(239, 68, 68, 0.2);
  }

  .rtss-dialog-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .rtss-btn {
    padding: 10px 16px;
    border-radius: 8px;
    font-size: calc(13px * var(--font-scale, 1));
    font-weight: 600;
    cursor: pointer;
    border: 1px solid transparent;
    transition: background 0.15s, opacity 0.15s;
  }

  .rtss-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .rtss-btn-primary {
    background: var(--accent-color, #66c0f4);
    color: #000;
    border-color: var(--accent-color, #66c0f4);
  }

  .rtss-btn-primary:hover:not(:disabled) {
    opacity: 0.9;
  }

  .rtss-btn-secondary {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary, #fff);
    border-color: var(--border-color, rgba(61, 68, 80, 0.3));
  }

  .rtss-btn-secondary:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.15);
  }

  .rtss-btn-ghost {
    background: transparent;
    color: var(--text-muted, #6b7280);
  }

  .rtss-btn-ghost:hover:not(:disabled) {
    color: var(--text-secondary, #9ca3af);
    background: rgba(255, 255, 255, 0.05);
  }

  .rtss-progress-bar {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
  }

  .rtss-progress-fill {
    height: 100%;
    background: var(--accent-color, #66c0f4);
    border-radius: 2px;
    transition: width 0.15s ease;
  }
</style>
