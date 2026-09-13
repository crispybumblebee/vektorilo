<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Button from "$lib/components/ui/button/Button.svelte";
  import Slider from "$lib/components/ui/slider/Slider.svelte";

  let dragActive = $state(false);
  let file: File | null = $state(null);
  let imageSrc: string | null = $state(null);

  let threshold = $state(128);
  let invert = $state(false);
  let removeBackground = $state(true);
  let detailLevel = $state(2); // 1: Low, 2: Medium, 3: High

  let canvas: HTMLCanvasElement | undefined = $state();
  let originalImageData: ImageData | null = null;
  let imageElement = new Image();

  let isExporting = $state(false);
  let errorMessage = $state<string | null>(null);

  // SVG Preview State
  let previewMode = $state<'raster' | 'vector'>('raster');
  let svgPreview = $state<string | null>(null);
  let isGeneratingPreview = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout>;

  function handleDragEnter(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragActive = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragActive = false;
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragActive = true;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragActive = false;

    if (
      e.dataTransfer &&
      e.dataTransfer.files &&
      e.dataTransfer.files.length > 0
    ) {
      handleFile(e.dataTransfer.files[0]);
    }
  }

  function handleFileInput(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      handleFile(target.files[0]);
    }
  }

  function handleFile(selectedFile: File) {
    errorMessage = null; // Reset errors
    if (!selectedFile.type.match("image/(jpeg|png|gif|tiff|bmp)")) {
      errorMessage =
        "Fehler: Dieses Dateiformat wird nicht unterstützt. Bitte nutze JPG, PNG, GIF, TIFF oder BMP.";
      return;
    }
    file = selectedFile;
    if (imageSrc) URL.revokeObjectURL(imageSrc);
    imageSrc = URL.createObjectURL(file);

    imageElement.onload = () => {
      initCanvas();
    };
    imageElement.src = imageSrc;
  }

  function initCanvas() {
    if (!canvas || !imageElement.src) return;

    // Scale canvas down to fit viewport nicely while maintaining aspect ratio
    const maxWidth = 800;
    const maxHeight = 600;
    let width = imageElement.width;
    let height = imageElement.height;

    if (width > maxWidth || height > maxHeight) {
      const ratio = Math.min(maxWidth / width, maxHeight / height);
      width = width * ratio;
      height = height * ratio;
    }

    canvas.width = width;
    canvas.height = height;

    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    if (!ctx) return;

    ctx.drawImage(imageElement, 0, 0, width, height);
    originalImageData = ctx.getImageData(0, 0, width, height);

    applyThreshold(threshold, invert);
  }

  function applyThreshold(thresh: number, inv: boolean) {
    if (!canvas || !originalImageData) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const newImageData = ctx.createImageData(
      originalImageData.width,
      originalImageData.height,
    );
    const data = originalImageData.data;
    const newData = newImageData.data;

    for (let i = 0; i < data.length; i += 4) {
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const a = data[i + 3];

      let v;
      if (a === 0) {
        // Transparent pixels become white background
        v = 255;
      } else {
        // Simple luminance calculation
        const luminance = 0.299 * r + 0.587 * g + 0.114 * b;
        v = luminance >= thresh ? 255 : 0;

        if (inv) {
          v = 255 - v;
        }
      }

      newData[i] = v;
      newData[i + 1] = v;
      newData[i + 2] = v;
      newData[i + 3] = (removeBackground && v === 255) ? 0 : 255;
    }

    ctx.putImageData(newImageData, 0, 0);
  }

  async function generateSvgPreview(thresh: number, inv: boolean, detail: number, removeBg: boolean) {
    if (!file) return;
    isGeneratingPreview = true;
    try {
      const arrayBuffer = await file.arrayBuffer();
      const bytes = new Uint8Array(arrayBuffer);
      const bytesArray = Array.from(bytes);

      let svgString = await invoke<string>("generate_svg_preview", {
        imageBytes: bytesArray,
        threshold: thresh,
        invert: inv,
        detailLevel: detail,
        removeBackground: removeBg,
      });

      // Inject viewBox to make it scale correctly with CSS
      const widthMatch = svgString.match(/width="([^"]+)"/);
      const heightMatch = svgString.match(/height="([^"]+)"/);
      
      if (widthMatch && heightMatch) {
        const w = parseFloat(widthMatch[1]);
        const h = parseFloat(heightMatch[1]);
        
        if (!svgString.includes("viewBox")) {
          svgString = svgString.replace('<svg ', `<svg viewBox="0 0 ${w} ${h}" `);
        }
        svgString = svgString.replace(/width="[^"]+"/, 'width="100%"');
        svgString = svgString.replace(/height="[^"]+"/, 'height="100%"');
      }

      svgPreview = svgString;
    } catch (e) {
      console.error(e);
    } finally {
      isGeneratingPreview = false;
    }
  }

  $effect(() => {
    // Re-apply threshold when slider or invert changes
    // Explicitly reading state here guarantees Svelte tracks the dependency
    const currentThreshold = threshold;
    const currentInvert = invert;
    const currentDetail = detailLevel;
    const currentMode = previewMode;
    const currentRemoveBg = removeBackground;
    
    // Always keep raster canvas updated (it's fast)
    applyThreshold(currentThreshold, currentInvert);

    // If in vector mode, debounce the heavy SVG generation
    if (currentMode === 'vector') {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        generateSvgPreview(currentThreshold, currentInvert, currentDetail, currentRemoveBg);
      }, 400);
    }
  });

  async function handleExport() {
    if (!file) return;
    isExporting = true;
    try {
      const arrayBuffer = await file.arrayBuffer();
      const bytes = new Uint8Array(arrayBuffer);
      const bytesArray = Array.from(bytes);

      await invoke("process_and_save_svg", {
        imageBytes: bytesArray,
        threshold: threshold,
        fileName: file.name,
        invert: invert,
        detailLevel: detailLevel,
        removeBackground: removeBackground,
      });

      alert("Export complete!");
    } catch (e) {
      console.error(e);
      alert("Error during export: " + e);
    } finally {
      isExporting = false;
    }
  }
</script>

<svelte:head>
  <title>Vektorilo</title>
</svelte:head>

<main
  class="min-h-screen p-8 flex flex-col items-center max-w-7xl mx-auto gap-8"
>
  <header class="text-center w-full mb-4">
    <h1 class="text-4xl font-bold text-foreground mb-2">Vektorilo</h1>
    <p class="text-slate-500">
      Convert raster images to Cricut-compatible SVGs instantly.
    </p>
  </header>

  <!-- Drag and Drop Zone -->
  {#if !imageSrc}
    <div
      role="button"
      tabindex="0"
      ondragenter={handleDragEnter}
      ondragleave={handleDragLeave}
      ondragover={handleDragOver}
      ondrop={handleDrop}
      onclick={() => document.getElementById("file-upload")?.click()}
      onkeydown={(e) =>
        e.key === "Enter" && document.getElementById("file-upload")?.click()}
      class="w-full max-w-2xl h-80 border-4 border-dashed rounded-3xl flex flex-col items-center justify-center transition-all cursor-pointer {dragActive
        ? 'border-primary bg-primary-light/50'
        : 'border-slate-200 hover:border-primary/50 hover:bg-slate-50'}"
    >
      <input
        id="file-upload"
        type="file"
        accept="image/png, image/jpeg, image/gif, image/tiff, image/bmp"
        class="hidden"
        onchange={handleFileInput}
      />

      <svg
        class="w-16 h-16 text-slate-400 mb-4 {dragActive
          ? 'text-primary'
          : ''}"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
        ></path>
      </svg>
      <p
        class="text-lg font-medium text-slate-600 {dragActive
          ? 'text-primary-dark'
          : ''}"
      >
        {dragActive ? "Bild hier loslassen" : "Bild hier hereinziehen"}
      </p>
      <p class="text-sm text-slate-400 mt-2">
        Unterstützt JPG, PNG, GIF, TIFF, BMP
      </p>
    </div>

    {#if errorMessage}
      <div
        class="mt-4 p-4 bg-red-50 text-red-600 rounded-xl border border-red-100 text-sm font-medium text-center w-full max-w-2xl shadow-sm"
      >
        {errorMessage}
      </div>
    {/if}
  {:else}
    <!-- Workspace -->
    <div
      class="w-full bg-card rounded-3xl shadow-sm border border-slate-100 p-8 flex flex-col lg:flex-row gap-8 items-start"
    >
      <!-- Canvas / SVG Preview (2/3 width) -->
      <div class="relative w-full lg:w-2/3 flex flex-col gap-4">
        <div class="relative w-full flex justify-center bg-slate-50 rounded-2xl border border-slate-100 overflow-hidden h-[400px] lg:h-[600px] checkerboard-bg">
          
          {#if previewMode === 'vector' && isGeneratingPreview}
            <div class="absolute inset-0 z-10 flex items-center justify-center bg-white/70 backdrop-blur-sm">
              <div class="w-10 h-10 border-4 border-primary/20 border-t-primary rounded-full animate-spin"></div>
            </div>
          {/if}

          <canvas 
            bind:this={canvas} 
            class="max-w-full max-h-full object-contain {previewMode === 'vector' ? 'hidden' : 'block'}"
          ></canvas>
          
          {#if previewMode === 'vector' && svgPreview}
            <div class="absolute inset-0 flex items-center justify-center p-4">
              <div class="w-full h-full flex items-center justify-center drop-shadow-md [&>svg]:max-w-full [&>svg]:max-h-full">
                {@html svgPreview}
              </div>
            </div>
          {/if}
        </div>

        <!-- Toggle Switch -->
        <div class="flex justify-center mt-2">
          <div class="bg-slate-100 p-1.5 rounded-2xl inline-flex text-sm font-semibold shadow-inner border border-slate-200/60">
            <button 
              onclick={() => { previewMode = 'raster'; }}
              class="px-5 py-2.5 rounded-xl transition-all {previewMode === 'raster' ? 'bg-white shadow-sm text-slate-800' : 'text-slate-500 hover:text-slate-700 hover:bg-slate-200/50'}"
            >
              Pixel-Vorschau (Schnell)
            </button>
            <button 
              onclick={() => { previewMode = 'vector'; }}
              class="px-5 py-2.5 rounded-xl transition-all {previewMode === 'vector' ? 'bg-white shadow-sm text-primary' : 'text-slate-500 hover:text-slate-700 hover:bg-slate-200/50'}"
            >
              Echtes SVG (Hohe Qualität)
            </button>
          </div>
        </div>
      </div>

      <!-- Controls (1/3 width) -->
      <div class="w-full lg:w-1/3 flex flex-col gap-6 sticky top-8">
        <!-- Threshold Slider -->
        <div
          class="flex flex-col gap-2 bg-slate-50 p-5 rounded-2xl border border-slate-100"
        >
          <div class="flex justify-between items-center mb-2">
            <label for="threshold" class="text-sm font-semibold text-slate-700"
              >Threshold (Schwarzanteil)</label
            >
            <span
              class="text-sm font-mono bg-white border border-slate-200 px-2 py-1 rounded-md text-slate-600 shadow-sm"
              >{threshold}</span
            >
          </div>
          <Slider min={0} max={255} bind:value={threshold} />
        </div>

        <!-- Additional Settings -->
        <div class="flex flex-col gap-4">
          <!-- Invert Toggle -->
          <div class="p-6 bg-slate-50 rounded-2xl border border-slate-100 flex flex-col gap-4">
            <label
              class="flex items-start gap-3 p-1 cursor-pointer group"
            >
              <input
                type="checkbox"
                bind:checked={invert}
                class="w-5 h-5 accent-primary text-primary rounded border-slate-300 focus:ring-primary cursor-pointer mt-0.5"
              />
              <div class="flex flex-col">
                <span class="text-sm font-semibold text-slate-700"
                  >Farben umkehren</span
                >
                <span class="text-xs text-slate-400"
                  >Wichtig bei schwarzen Hintergründen</span
                >
              </div>
            </label>

            <label
              class="flex items-start gap-3 p-1 cursor-pointer group"
            >
              <input
                type="checkbox"
                bind:checked={removeBackground}
                class="w-5 h-5 accent-primary text-primary rounded border-slate-300 focus:ring-primary cursor-pointer mt-0.5"
              />
              <div class="flex flex-col">
                <span class="text-sm font-semibold text-slate-700"
                  >Hintergrund entfernen</span
                >
                <span class="text-xs text-slate-400"
                  >Nur das Motiv transparent plotten</span
                >
              </div>
            </label>
          </div>

          <!-- Detail Level Select -->
          <div
            class="flex flex-col justify-center p-4 border border-slate-200 rounded-2xl"
          >
            <label
              for="detail"
              class="text-xs font-semibold text-slate-500 mb-2"
              >Detailgrad (Glättung)</label
            >
            <select
              id="detail"
              bind:value={detailLevel}
              class="w-full text-sm font-medium bg-transparent outline-none cursor-pointer text-slate-700"
            >
              <option value={1}>Niedrig (Sehr glatt, gut für Vinyl)</option>
              <option value={2}>Mittel (Standard)</option>
              <option value={3}>Hoch (Exakt, Print-then-cut)</option>
            </select>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex flex-col gap-3 mt-2 pt-4 border-t border-slate-100">
          <Button
            onclick={handleExport}
            disabled={isExporting}
            class="w-full text-lg h-12"
          >
            {isExporting ? "Exportiere..." : "SVG Exportieren"}
          </Button>
          <Button
            variant="ghost"
            onclick={() => {
              imageSrc = null;
              file = null;
              originalImageData = null;
            }}
            class="w-full"
          >
            Anderes Bild wählen
          </Button>
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  .checkerboard-bg {
    background-image: 
      linear-gradient(45deg, #f0f0f0 25%, transparent 25%), 
      linear-gradient(-45deg, #f0f0f0 25%, transparent 25%), 
      linear-gradient(45deg, transparent 75%, #f0f0f0 75%), 
      linear-gradient(-45deg, transparent 75%, #f0f0f0 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
    background-color: #fafafa;
  }
</style>
