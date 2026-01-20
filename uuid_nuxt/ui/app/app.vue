<template>
  <section class="relative min-h-screen w-full overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
    <!-- Animated background elements -->
    <div class="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PGRlZnM+PHBhdHRlcm4gaWQ9ImdyaWQiIHdpZHRoPSI2MCIgaGVpZ2h0PSI2MCIgcGF0dGVyblVuaXRzPSJ1c2VyU3BhY2VPblVzZSI+PHBhdGggZD0iTSAxMCAwIEwgMCAwIDAgMTAiIGZpbGw9Im5vbmUiIHN0cm9rZT0icmdiYSgyNTUsMjU1LDI1NSwwLjAzKSIgc3Ryb2tlLXdpZHRoPSIxIi8+PC9wYXR0ZXJuPjwvZGVmcz48cmVjdCB3aWR0aD0iMTAwJSIgaGVpZ2h0PSIxMDAlIiBmaWxsPSJ1cmwoI2dyaWQpIi8+PC9zdmc+')] opacity-40"></div>
    
    <div class="absolute top-1/4 left-1/4 w-96 h-96 bg-cyan-500/20 rounded-full blur-[120px] animate-pulse"></div>
    <div class="absolute bottom-1/4 right-1/4 w-96 h-96 bg-fuchsia-500/20 rounded-full blur-[120px] animate-pulse" style="animation-delay: 1s;"></div>
    
    <!-- Main content -->
    <div class="relative z-10 container mx-auto px-4 py-12 max-w-5xl">
      <!-- Header -->
      <div class="text-center mb-16 animate-fade-in-down">
        <div class="inline-flex items-center gap-3 mb-4">
          <div class="w-2 h-2 bg-cyan-400 rounded-full animate-ping"></div>
          <h1 class="text-7xl md:text-8xl font-black tracking-tighter">
            <span class="bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-cyan-400 bg-clip-text text-transparent bg-[length:200%_auto] animate-gradient">
              UUID
            </span>
          </h1>
          <div class="w-2 h-2 bg-fuchsia-400 rounded-full animate-ping" style="animation-delay: 0.3s;"></div>
        </div>
        <p class="text-slate-400 text-lg font-light tracking-wide">Universal Unique Identifier Generator</p>
        <div class="mt-2 h-px w-32 mx-auto bg-gradient-to-r from-transparent via-cyan-500 to-transparent"></div>
        
        <!-- Info Badge -->
        <div class="mt-6 inline-flex items-center gap-2 px-4 py-2 rounded-full bg-amber-500/10 border border-amber-500/30 text-amber-400 text-sm">
          <svg class="w-4 h-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <span class="font-medium">Generated files auto-delete after 10 minutes</span>
        </div>
      </div>

      <!-- Input Card -->
      <div class="backdrop-blur-xl bg-slate-900/50 border border-slate-800/50 rounded-3xl p-8 mb-8 shadow-2xl shadow-cyan-500/10 hover:shadow-cyan-500/20 transition-all duration-500 animate-fade-in-up">
        <div class="space-y-6">
          <!-- Input Group -->
          <div>
            <label for="uuid-input" class="flex items-center justify-between mb-3 text-sm font-medium text-slate-300">
              <span class="flex items-center gap-2">
                <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 20l4-16m2 16l4-16M6 9h14M4 15h14"/>
                </svg>
                Quantity
              </span>
              <span class="text-fuchsia-400 font-mono text-xs">{{ uuid }} UUID{{ uuid !== 1 ? 's' : '' }}</span>
            </label>
            
            <div class="relative group">
              <input 
                id="uuid-input"
                type="number" 
                v-model="uuid" 
                class="w-full px-6 py-4 bg-slate-950/80 border border-slate-700 rounded-2xl text-slate-100 font-mono text-lg placeholder:text-slate-600 focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500/50 transition-all duration-300"
                placeholder="Enter quantity (1-50000)"
                min="1"
                max="50000"
              >
              <div class="absolute inset-0 rounded-2xl bg-gradient-to-r from-cyan-500/20 to-fuchsia-500/20 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none -z-10 blur-xl"></div>
            </div>
          </div>

          <!-- Action Buttons -->
          <div class="flex flex-col sm:flex-row gap-3">
            <button 
              @click="get_data" 
              :disabled="loading"
              class="relative flex-1 group overflow-hidden px-8 py-4 bg-gradient-to-r from-cyan-600 to-cyan-500 hover:from-cyan-500 hover:to-cyan-400 text-white font-semibold rounded-2xl shadow-lg shadow-cyan-500/30 hover:shadow-cyan-500/50 transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed transform hover:-translate-y-0.5"
            >
              <span class="relative z-10 flex items-center justify-center gap-2">
                <svg v-if="!loading" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
                </svg>
                <svg v-else class="w-5 h-5 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                </svg>
                {{ loading ? 'Generating...' : 'Generate UUIDs' }}
              </span>
              <div class="absolute inset-0 bg-gradient-to-r from-cyan-400 to-fuchsia-400 opacity-0 group-hover:opacity-20 transition-opacity duration-300"></div>
            </button>

            <a 
              v-if="filename" 
              :href="`/get-file/${filename}`" 
              download
              class="relative flex-1 group overflow-hidden px-8 py-4 bg-gradient-to-r from-fuchsia-600 to-fuchsia-500 hover:from-fuchsia-500 hover:to-fuchsia-400 text-white font-semibold rounded-2xl shadow-lg shadow-fuchsia-500/30 hover:shadow-fuchsia-500/50 transition-all duration-300 transform hover:-translate-y-0.5 text-center"
            >
              <span class="relative z-10 flex items-center justify-center gap-2">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                </svg>
                Download File
              </span>
              <div class="absolute inset-0 bg-gradient-to-r from-fuchsia-400 to-cyan-400 opacity-0 group-hover:opacity-20 transition-opacity duration-300"></div>
            </a>
          </div>
        </div>
      </div>

      <!-- Results Section -->
      <transition
        enter-active-class="transition-all duration-500 ease-out"
        enter-from-class="opacity-0 translate-y-8"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition-all duration-300 ease-in"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 -translate-y-8"
      >
        <div v-if="uuids.length" class="backdrop-blur-xl bg-slate-900/50 border border-slate-800/50 rounded-3xl overflow-hidden shadow-2xl shadow-fuchsia-500/10">
          <!-- Results Header -->
          <div class="flex items-center justify-between p-6 border-b border-slate-800/50 bg-gradient-to-r from-slate-900/80 to-slate-900/50">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-cyan-500 to-fuchsia-500 flex items-center justify-center">
                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                </svg>
              </div>
              <div>
                <h2 class="text-xl font-bold text-slate-100">Generated UUIDs</h2>
                <p class="text-sm text-slate-400">Click any UUID to copy</p>
              </div>
            </div>
            
            <div class="flex items-center gap-3">
              <span class="px-4 py-2 rounded-full bg-cyan-500/10 border border-cyan-500/30 text-cyan-400 text-sm font-mono font-semibold">
                {{ uuids.length }} items
              </span>
              <button 
                @click="copyAll"
                class="px-4 py-2 rounded-xl bg-slate-800/50 hover:bg-slate-800 border border-slate-700 text-slate-300 hover:text-white transition-all duration-200 text-sm font-medium flex items-center gap-2 group"
              >
                <svg v-if="!copied" class="w-4 h-4 group-hover:scale-110 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                </svg>
                <svg v-else class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                </svg>
                {{ copied ? 'Copied!' : 'Copy All' }}
              </button>
            </div>
          </div>

          <!-- UUID List -->
          <div class="p-6 max-h-[600px] overflow-y-auto custom-scrollbar">
            <div class="space-y-2">
              <div 
                v-for="(item, index) in uuids" 
                :key="index"
                @click="copyUuid(item, index)"
                :style="{ animationDelay: `${index * 20}ms` }"
                class="group relative flex items-center gap-4 p-4 rounded-xl bg-slate-950/50 hover:bg-slate-950/80 border border-slate-800/30 hover:border-cyan-500/30 cursor-pointer transition-all duration-200 animate-slide-in-left"
              >
                <!-- Index -->
                <span class="flex-shrink-0 w-12 text-center text-slate-600 group-hover:text-cyan-400 font-mono text-sm font-semibold transition-colors">
                  {{ String(index + 1).padStart(3, '0') }}
                </span>

                <!-- UUID Value -->
                <code class="flex-1 text-slate-300 group-hover:text-slate-100 font-mono text-sm tracking-wide transition-colors">
                  {{ item }}
                </code>

                <!-- Copy Indicator -->
                <span class="flex-shrink-0 text-xs font-medium transition-all duration-200"
                  :class="copiedIndex === index ? 'text-green-400 scale-110' : 'text-slate-600 group-hover:text-cyan-400 opacity-0 group-hover:opacity-100'">
                  {{ copiedIndex === index ? '✓ Copied!' : 'Click to copy' }}
                </span>

                <!-- Hover Effect -->
                <div class="absolute inset-0 rounded-xl bg-gradient-to-r from-cyan-500/0 via-cyan-500/5 to-fuchsia-500/0 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none"></div>
              </div>
            </div>
          </div>
        </div>
      </transition>

      <!-- Empty State -->
      <div v-if="!uuids.length && !loading" class="text-center py-16 animate-fade-in">
        <div class="inline-flex items-center justify-center w-20 h-20 rounded-full bg-slate-800/50 border border-slate-700 mb-4">
          <svg class="w-10 h-10 text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"/>
          </svg>
        </div>
        <p class="text-slate-500 text-lg">Generate UUIDs to get started</p>
      </div>
    </div>
  </section>
</template>

<script setup>
const uuid = ref(10);
const uuids = ref([]);
const filename = ref(null);
const loading = ref(false);
const copied = ref(false);
const copiedIndex = ref(null);

async function get_data() {
  if (uuid.value > 50000) {
    alert('Maximum limit is 50,000 UUIDs');
    uuid.value = 50000;
    return;
  }
  
  loading.value = true;
  try {
    let data = await $fetch(`/api/generate/${uuid.value}`);
    uuids.value = data.uuids;
    filename.value = data.file;
  } catch (error) {
    console.error('Error generating UUIDs:', error);
  } finally {
    loading.value = false;
  }
}

async function copyAll() {
  try {
    await navigator.clipboard.writeText(uuids.value.join('\n'));
    copied.value = true;
    setTimeout(() => copied.value = false, 2000);
  } catch (error) {
    console.error('Failed to copy:', error);
  }
}

async function copyUuid(item, index) {
  try {
    await navigator.clipboard.writeText(item);
    copiedIndex.value = index;
    setTimeout(() => copiedIndex.value = null, 2000);
  } catch (error) {
    console.error('Failed to copy:', error);
  }
}
</script>

<style scoped>
/* Custom animations */
@keyframes gradient {
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
}

@keyframes fade-in-down {
  from {
    opacity: 0;
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slide-in-left {
  from {
    opacity: 0;
    transform: translateX(-20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.animate-gradient {
  animation: gradient 6s ease infinite;
}

.animate-fade-in-down {
  animation: fade-in-down 0.8s ease-out;
}

.animate-fade-in-up {
  animation: fade-in-up 0.8s ease-out 0.2s both;
}

.animate-fade-in {
  animation: fade-in 0.6s ease-out;
}

.animate-slide-in-left {
  animation: slide-in-left 0.4s ease-out both;
}

/* Custom scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(15, 23, 42, 0.5);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: linear-gradient(to bottom, #06b6d4, #d946ef);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(to bottom, #22d3ee, #e879f9);
}

/* Remove number input arrows */
input[type="number"]::-webkit-inner-spin-button,
input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

input[type="number"] {
  -moz-appearance: textfield;
}
</style>