<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { Library } from '$lib/types/library';
  import type {
    BilibiliAccount,
    BilibiliUploader,
    BilibiliVideo,
    BilibiliDownload,
    DownloadProgress,
    BilibiliSettings
  } from '$lib/types/bilibili';
  import { listLibraries, getCurrentLibrary, createLibrary, setCurrentLibrary } from '$lib/services/library';
  import * as bili from '$lib/services/bilibili';

  // 库状态
  let libraries: Library[] = [];
  let currentLibrary: Library | null = null;

  // 账号
  let account: BilibiliAccount | null = null;
  let loginStep: 'idle' | 'qrcode' | 'scanning' | 'expired' = 'idle';
  let qrcodeUrl = '';
  let qrcodeKey = '';
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  // UP 主
  let uploaders: BilibiliUploader[] = [];
  let selectedUploader: BilibiliUploader | null = null;
  let addUploaderMid = '';
  let addingUploader = false;

  // 视频
  let videos: BilibiliVideo[] = [];
  let selectedVideoIds = new Set<number>();
  let videoLoading = false;
  let syncingVideos = false;
  let videoViewMode: 'list' | 'cover' = 'list';

  // 下载
  let downloads: BilibiliDownload[] = [];
  let downloadProgress: DownloadProgress | null = null;
  let progressTimer: ReturnType<typeof setInterval> | null = null;
  let isDownloading = false;

  // 设置
  let settings: BilibiliSettings | null = null;
  let downloadDir = '';

  // 搜索
  let searchQuery = '';

  async function initLibrary() {
    try {
      libraries = await listLibraries('bilibili');
      if (libraries.length === 0) {
        await createLibrary({
          name: 'B站音频',
          moduleType: 'bilibili',
          storagePath: 'bilibili',
          description: 'B站视频音频下载'
        });
        libraries = await listLibraries('bilibili');
      }
      currentLibrary = await getCurrentLibrary('bilibili').catch(() => libraries[0] || null);
      if (currentLibrary) {
        await loadUploaders();
      }
    } catch (e) {
      console.error('[bilibili] initLibrary 失败:', e);
    }
  }

  onMount(async () => {
    try {
      settings = await bili.getSettings();
      downloadDir = settings.downloadDir;
    } catch (e) {
      console.error('[bilibili] getSettings 失败:', e);
    }

    try {
      account = await bili.getAccount();
    } catch (e) {
      console.error('[bilibili] getAccount 失败:', e);
    }

    await initLibrary();
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
    if (progressTimer) clearInterval(progressTimer);
  });

  // --- 登录 ---

  async function startLogin() {
    try {
      const qr = await bili.generateQrcode();
      qrcodeUrl = qr.url;
      qrcodeKey = qr.qrcodeKey;
      loginStep = 'qrcode';

      pollTimer = setInterval(async () => {
        try {
          const status = await bili.pollQrcode(qrcodeKey);
          if (status.code === 0) {
            account = status.account ?? null;
            loginStep = 'idle';
            if (pollTimer) clearInterval(pollTimer);
            await initLibrary();
          } else if (status.code === 86090) {
            loginStep = 'scanning';
          } else if (status.code === 86038) {
            loginStep = 'expired';
            if (pollTimer) clearInterval(pollTimer);
          }
        } catch {
          // 忽略轮询错误
        }
      }, 2000);
    } catch (e) {
      alert(`生成二维码失败: ${e}`);
    }
  }

  async function handleLogout() {
    if (!confirm('确定退出登录？')) return;
    await bili.logout();
    account = null;
  }

  // --- UP 主 ---

  async function loadUploaders() {
    if (!currentLibrary) return;
    uploaders = await bili.listUploaders(currentLibrary.id);
  }

  async function handleAddUploader() {
    if (!currentLibrary || !addUploaderMid.trim()) return;
    const mid = parseInt(addUploaderMid.trim());
    if (isNaN(mid)) {
      alert('请输入有效的 UP 主 UID');
      return;
    }
    addingUploader = true;
    try {
      await bili.addUploader(currentLibrary.id, mid);
      await loadUploaders();
      addUploaderMid = '';
    } catch (e) {
      alert(`添加失败: ${e}`);
    } finally {
      addingUploader = false;
    }
  }

  async function handleRemoveUploader(uploader: BilibiliUploader) {
    if (!currentLibrary || !confirm(`确定移除 ${uploader.name}？`)) return;
    await bili.removeUploader(currentLibrary.id, uploader.id);
    if (selectedUploader?.id === uploader.id) {
      selectedUploader = null;
      videos = [];
    }
    await loadUploaders();
  }

  async function selectUploader(uploader: BilibiliUploader) {
    selectedUploader = uploader;
    selectedVideoIds = new Set();
    searchQuery = '';
    await loadVideos();
  }

  // --- 视频 ---

  async function loadVideos() {
    if (!currentLibrary || !selectedUploader) return;
    videoLoading = true;
    try {
      videos = await bili.listVideos(currentLibrary.id, selectedUploader.id);
    } finally {
      videoLoading = false;
    }
  }

  async function handleSyncVideos() {
    if (!currentLibrary || !selectedUploader) return;
    syncingVideos = true;
    try {
      videos = await bili.syncVideos(currentLibrary.id, selectedUploader.id);
      await loadUploaders();
    } catch (e) {
      alert(`同步失败: ${e}`);
    } finally {
      syncingVideos = false;
    }
  }

  function toggleVideo(id: number) {
    const next = new Set(selectedVideoIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedVideoIds = next;
  }

  function selectAll() {
    selectedVideoIds = new Set(filteredVideos.map((v: BilibiliVideo) => v.id));
  }

  function deselectAll() {
    selectedVideoIds = new Set<number>();
  }

  // --- 下载 ---

  async function handleDownload() {
    if (!currentLibrary || selectedVideoIds.size === 0) return;
    if (!settings?.ytdlpInstalled) {
      alert('请先安装 yt-dlp：https://github.com/yt-dlp/yt-dlp#installation');
      return;
    }
    isDownloading = true;
    try {
      await bili.startDownload(
        currentLibrary.id,
        Array.from(selectedVideoIds),
        downloadDir
      );
      selectedVideoIds = new Set<number>();
      downloads = await bili.listDownloads(currentLibrary.id);
      startProgressPolling();
    } catch (e) {
      alert(`下载失败: ${e}`);
      isDownloading = false;
    }
  }

  function startProgressPolling() {
    if (progressTimer) clearInterval(progressTimer);
    let pollCount = 0;
    progressTimer = setInterval(async () => {
      pollCount++;
      downloadProgress = await bili.getDownloadProgress();
      if (currentLibrary) {
        downloads = await bili.listDownloads(currentLibrary.id);
      }
      const hasActive = downloads.some(d => d.status === 'pending' || d.status === 'downloading');
      if (!hasActive && !downloadProgress && pollCount > 3) {
        isDownloading = false;
        if (progressTimer) clearInterval(progressTimer);
      }
    }, 1000);
  }

  async function handleCancelDownload() {
    await bili.cancelDownload();
    isDownloading = false;
    if (progressTimer) clearInterval(progressTimer);
  }

  // --- 库切换 ---

  async function handleLibrarySwitch(e: Event) {
    const target = e.target as HTMLSelectElement;
    const id = parseInt(target.value);
    await setCurrentLibrary(id);
    currentLibrary = libraries.find(l => l.id === id) ?? null;
    selectedUploader = null;
    videos = [];
    selectedVideoIds = new Set();
    await loadUploaders();
  }

  // --- 过滤 ---

  $: filteredVideos = searchQuery
    ? videos.filter(v => v.title.toLowerCase().includes(searchQuery.toLowerCase()))
    : videos;
</script>

<div class="h-full flex flex-col bg-gray-50">
  <!-- 顶部栏 -->
  <div class="flex items-center gap-4 px-6 py-3 bg-white border-b">
    <h1 class="text-xl font-bold text-pink-600">B站音频</h1>

    <select
      class="px-3 py-1.5 border rounded-lg text-sm"
      value={currentLibrary?.id ?? ''}
      on:change={handleLibrarySwitch}
    >
      {#each libraries as lib}
        <option value={lib.id}>{lib.name}</option>
      {/each}
    </select>

    <div class="flex-1"></div>

    {#if account}
      <div class="flex items-center gap-2 text-sm">
        {#if account.avatarUrl}
          <img src={account.avatarUrl} alt="" class="w-6 h-6 rounded-full" referrerpolicy="no-referrer" />
        {/if}
        <span class="text-gray-700">{account.username}</span>
        <button class="text-gray-400 hover:text-red-500 text-xs" on:click={handleLogout}>退出</button>
      </div>
    {:else}
      <button
        class="px-4 py-1.5 bg-pink-500 text-white rounded-lg text-sm hover:bg-pink-600"
        on:click={startLogin}
      >
        扫码登录
      </button>
    {/if}
  </div>

  <!-- 登录弹窗 -->
  {#if loginStep !== 'idle'}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white rounded-xl p-8 text-center max-w-sm">
        <h2 class="text-lg font-bold mb-4">B站扫码登录</h2>
        {#if loginStep === 'expired'}
          <p class="text-red-500 mb-4">二维码已过期</p>
          <button class="px-4 py-2 bg-pink-500 text-white rounded-lg" on:click={startLogin}>重新生成</button>
        {:else}
          <div class="mb-4">
            <img
              src="https://api.qrserver.com/v1/create-qr-code/?size=200x200&data={encodeURIComponent(qrcodeUrl)}"
              alt="QR Code"
              class="mx-auto w-48 h-48"
            />
          </div>
          <p class="text-sm text-gray-500">
            {loginStep === 'scanning' ? '已扫码，请在手机上确认' : '请使用 B 站手机 App 扫码'}
          </p>
        {/if}
        <button
          class="mt-4 text-sm text-gray-400 hover:text-gray-600"
          on:click={() => { loginStep = 'idle'; if (pollTimer) clearInterval(pollTimer); }}
        >
          取消
        </button>
      </div>
    </div>
  {/if}

  <!-- 主内容 -->
  <div class="flex-1 flex overflow-hidden">
    {#if !account}
      <!-- 未登录居中提示 -->
      <div class="flex-1 flex flex-col items-center justify-center gap-4">
        <div class="text-6xl">📺</div>
        <p class="text-gray-500">请先登录 B 站账号</p>
        <button
          class="px-6 py-2.5 bg-pink-500 text-white rounded-lg text-base hover:bg-pink-600"
          on:click={startLogin}
        >
          扫码登录
        </button>
      </div>
    {:else}
    <!-- 左侧：UP 主列表 -->
    <div class="w-64 bg-white border-r flex flex-col">
      <div class="p-3 border-b">
        <div class="flex gap-2">
          <input
            type="text"
            placeholder="UP 主 UID"
            bind:value={addUploaderMid}
            class="flex-1 px-2 py-1.5 border rounded text-sm"
            on:keydown={(e) => e.key === 'Enter' && handleAddUploader()}
          />
          <button
            class="px-3 py-1.5 bg-pink-500 text-white rounded text-sm disabled:opacity-50"
            on:click={handleAddUploader}
            disabled={addingUploader}
          >
            {addingUploader ? '...' : '+'}
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto">
        {#each uploaders as uploader}
          <div
            class="w-full flex items-center gap-2 px-3 py-2.5 text-left hover:bg-gray-50 border-b cursor-pointer
              {selectedUploader?.id === uploader.id ? 'bg-pink-50 border-l-2 border-l-pink-500' : ''}"
            role="button"
            tabindex="0"
            on:click={() => selectUploader(uploader)}
            on:keydown={(e) => e.key === 'Enter' && selectUploader(uploader)}
          >
            {#if uploader.faceUrl}
              <img src={uploader.faceUrl} alt="" class="w-8 h-8 rounded-full flex-shrink-0" referrerpolicy="no-referrer" />
            {:else}
              <div class="w-8 h-8 rounded-full bg-gray-200 flex-shrink-0"></div>
            {/if}
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium truncate">{uploader.name}</div>
              <div class="text-xs text-gray-400">{uploader.totalVideos} 个视频</div>
            </div>
            <button
              class="text-gray-300 hover:text-red-500 text-xs p-1"
              on:click|stopPropagation={() => handleRemoveUploader(uploader)}
            >✕</button>
          </div>
        {/each}
        {#if uploaders.length === 0}
          <div class="p-4 text-center text-sm text-gray-400">
            添加 UP 主开始使用
          </div>
        {/if}
      </div>
    </div>

    <!-- 右侧：视频列表 -->
    <div class="flex-1 flex flex-col">
      {#if selectedUploader}
        <!-- 工具栏 -->
        <div class="flex flex-wrap items-center gap-2 px-4 py-2 bg-white border-b">
          <span class="font-medium text-sm">{selectedUploader.name}</span>
          <button
            class="px-2 py-1 text-xs border rounded hover:bg-gray-50 disabled:opacity-50"
            on:click={handleSyncVideos}
            disabled={syncingVideos}
          >
            {syncingVideos ? '同步中...' : '同步'}
          </button>

          <input
            type="text"
            placeholder="搜索..."
            bind:value={searchQuery}
            class="px-2 py-1 border rounded text-xs w-32"
          />

          <button class="text-xs text-pink-600 hover:underline" on:click={selectAll}>全选</button>
          <button class="text-xs text-gray-500 hover:underline" on:click={deselectAll}>取消</button>

          <span class="text-xs text-gray-500">已选{selectedVideoIds.size}</span>

          <div class="flex border rounded overflow-hidden">
            <button
              class="px-2 py-1 text-xs {videoViewMode === 'list' ? 'bg-pink-500 text-white' : 'bg-white text-gray-600 hover:bg-gray-50'}"
              on:click={() => videoViewMode = 'list'}
              title="列表"
            >☰</button>
            <button
              class="px-2 py-1 text-xs {videoViewMode === 'cover' ? 'bg-pink-500 text-white' : 'bg-white text-gray-600 hover:bg-gray-50'}"
              on:click={() => videoViewMode = 'cover'}
              title="封面"
            >▦</button>
          </div>

          <button
            class="px-3 py-1 bg-pink-500 text-white rounded text-xs hover:bg-pink-600 disabled:opacity-50"
            on:click={handleDownload}
            disabled={selectedVideoIds.size === 0 || isDownloading}
          >
            {isDownloading ? '下载中...' : '下载音频'}
          </button>
        </div>

        <!-- 视频列表 -->
        <div class="flex-1 overflow-y-auto">
          {#if videoLoading || syncingVideos}
            <div class="p-8 text-center text-gray-400">
              {syncingVideos ? '正在从 B 站同步视频列表，请稍候...' : '加载中...'}
            </div>
          {:else if filteredVideos.length === 0}
            <div class="p-8 text-center text-gray-400">
              {videos.length === 0 ? '点击「同步视频列表」获取视频' : '没有匹配的视频'}
            </div>
          {:else if videoViewMode === 'list'}
            {#each filteredVideos as video}
              <label
                class="flex items-center gap-3 px-4 py-3 border-b hover:bg-gray-50 cursor-pointer
                  {selectedVideoIds.has(video.id) ? 'bg-pink-50' : ''}"
              >
                <input
                  type="checkbox"
                  checked={selectedVideoIds.has(video.id)}
                  on:change={() => toggleVideo(video.id)}
                  class="w-4 h-4 accent-pink-500"
                />
                {#if video.coverUrl}
                  <img src={video.coverUrl} alt="" class="w-24 h-14 object-cover rounded flex-shrink-0" referrerpolicy="no-referrer" />
                {/if}
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium truncate">{video.title}</div>
                  <div class="text-xs text-gray-400 mt-1">
                    {bili.formatDuration(video.duration)}
                    · {bili.formatPlayCount(video.playCount)} 播放
                    {#if video.publishTime}
                      · {new Date(video.publishTime * 1000).toLocaleDateString()}
                    {/if}
                  </div>
                </div>
              </label>
            {/each}
          {:else}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3 p-4">
              {#each filteredVideos as video}
                <label
                  class="relative rounded-lg overflow-hidden border cursor-pointer
                    {selectedVideoIds.has(video.id) ? 'ring-2 ring-pink-500' : ''}"
                >
                  <input
                    type="checkbox"
                    checked={selectedVideoIds.has(video.id)}
                    on:change={() => toggleVideo(video.id)}
                    class="absolute top-2 left-2 w-4 h-4 accent-pink-500 z-10"
                  />
                  {#if video.coverUrl}
                    <img src={video.coverUrl} alt="" class="w-full aspect-video object-cover" referrerpolicy="no-referrer" />
                  {:else}
                    <div class="w-full aspect-video bg-gray-200 flex items-center justify-center">📺</div>
                  {/if}
                  <div class="p-2">
                    <div class="text-xs font-medium truncate">{video.title}</div>
                    <div class="text-xs text-gray-400 mt-0.5">
                      {bili.formatDuration(video.duration)} · {bili.formatPlayCount(video.playCount)}
                    </div>
                  </div>
                </label>
              {/each}
            </div>
          {/if}
        </div>
      {:else}
        <div class="flex-1 flex items-center justify-center text-gray-400">
          从左侧选择一个 UP 主
        </div>
      {/if}

      <!-- 下载进度 -->
      {#if isDownloading || downloads.some(d => d.status === 'downloading' || d.status === 'pending')}
        <div class="border-t bg-white px-4 py-3">
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-medium">下载队列</span>
            <button class="text-xs text-red-500 hover:underline" on:click={handleCancelDownload}>取消</button>
          </div>
          {#if downloadProgress}
            <div class="mb-2">
              <div class="flex items-center justify-between text-xs text-gray-500 mb-1">
                <span>{downloadProgress.bvid}</span>
                <span>{downloadProgress.percent.toFixed(1)}% · {downloadProgress.speed} · ETA {downloadProgress.eta}</span>
              </div>
              <div class="w-full bg-gray-200 rounded-full h-2">
                <div
                  class="bg-pink-500 h-2 rounded-full transition-all"
                  style="width: {downloadProgress.percent}%"
                ></div>
              </div>
            </div>
          {/if}
          <div class="text-xs text-gray-400">
            {downloads.filter(d => d.status === 'completed').length} 完成 /
            {downloads.filter(d => d.status === 'pending').length} 等待中 /
            {downloads.filter(d => d.status === 'failed').length} 失败
          </div>
        </div>
      {/if}
    </div>
    {/if}
  </div>
</div>
