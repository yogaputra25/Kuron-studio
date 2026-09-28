import { derived, writable } from "svelte/store";

export type Lang = "en" | "id" | "zh";
export type Theme = "dark" | "light";

function stored(key: string): string | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage.getItem(key);
  } catch {
    return null; // SSR / vitest / private mode tanpa storage.
  }
}

const storedLang = stored("kuron-lang") as Lang | null;
const storedTheme = stored("kuron-theme") as Theme | null;

export const lang = writable<Lang>(
  storedLang === "en" || storedLang === "id" || storedLang === "zh" ? storedLang : "id",
);
export const theme = writable<Theme>(storedTheme === "light" ? "light" : "dark");

lang.subscribe((v) => {
  try {
    localStorage.setItem("kuron-lang", v);
  } catch {
    /* private mode */
  }
});
theme.subscribe((v) => {
  try {
    localStorage.setItem("kuron-theme", v);
  } catch {
    /* private mode */
  }
  if (typeof document !== "undefined") {
    document.documentElement.dataset.theme = v;
  }
});

// ponytail: flat keys, no nesting/plural rules. Add a lang by copying one block.
const STR: Record<string, Record<Lang, string>> = {
  providers: { en: "Providers", id: "Providers", zh: "服务商" },
  batch: { en: "Batch", id: "Batch", zh: "批量" },
  glossary: { en: "Glossary", id: "Glossary", zh: "术语表" },
  review: { en: "Review", id: "Review", zh: "审阅" },
  detectAll: { en: "Detect all", id: "Detect semua", zh: "全部检测" },
  importFolder: { en: "Import folder", id: "Import folder", zh: "导入文件夹" },
  importFiles: { en: "Import files", id: "Import files", zh: "导入文件" },
  close: { en: "Close", id: "Tutup", zh: "关闭" },
  newProject: { en: "Create first project", id: "Buat project pertama", zh: "创建首个项目" },
  projectName: { en: "Project name…", id: "Nama project…", zh: "项目名称…" },
  create: { en: "Create", id: "Buat", zh: "创建" },
  emptyHint: {
    en: "Drag-drop a folder/zip here, or use Import.",
    id: "Drag-drop folder/zip ke sini, atau pakai tombol Import.",
    zh: "将文件夹/压缩包拖到此处，或使用导入按钮。",
  },
  batchTitle: { en: "Batch translate", id: "Batch translate", zh: "批量翻译" },
  parallel: { en: "parallel", id: "paralel", zh: "并行" },
  failed: { en: "failed", id: "gagal", zh: "失败" },
  retryFailed: { en: "Retry failed", id: "Retry yang gagal", zh: "重试失败项" },
  running: { en: "Running…", id: "Jalan…", zh: "运行中…" },
  done: { en: "done", id: "selesai", zh: "完成" },
  pages: { en: "pages", id: "halaman", zh: "页" },
  bubbles: { en: "bubbles", id: "bubble", zh: "气泡" },
  original: { en: "Original", id: "Asli", zh: "原文" },
  reading: { en: "Reading", id: "Bacaan", zh: "读音" },
  translated: { en: "Translated", id: "Terjemahan", zh: "译文" },
  add: { en: "Add", id: "Tambah", zh: "添加" },
  edit: { en: "Edit", id: "Edit", zh: "编辑" },
  save: { en: "Save", id: "Simpan", zh: "保存" },
  remove: { en: "Remove", id: "Hapus", zh: "删除" },
  export: { en: "Export", id: "Export", zh: "导出" },
  share: { en: "Share", id: "Bagikan", zh: "分享" },
  search: { en: "Search", id: "Cari", zh: "搜索" },
  exportJson: { en: "Export JSON", id: "Export JSON", zh: "导出 JSON" },
  exportPng: { en: "Export PNG", id: "Export PNG", zh: "导出 PNG" },
  exportCbz: { en: "Export CBZ", id: "Export CBZ", zh: "导出 CBZ" },
  refresh: { en: "Refresh", id: "Refresh", zh: "刷新" },
  cleanImages: { en: "Clean broken", id: "Bersihkan rusak", zh: "清理损坏" },
  deleteProject: { en: "Delete project", id: "Hapus project", zh: "删除项目" },
  confirmDeleteProject: {
    en: "Delete this project and its imported pages? Source images on disk are kept.",
    id: "Hapus project ini beserta halamannya? File gambar asli di disk tetap disimpan.",
    zh: "删除该项目及其页面？磁盘上的原图会保留。",
  },
  confirmClean: {
    en: "Remove pages whose file is missing or undecodable? Copies inside the app folder are deleted too.",
    id: "Buang halaman yang file-nya hilang atau rusak? Salinan di dalam folder app juga dihapus.",
    zh: "移除文件缺失或损坏的页面？应用目录内的副本也会被删除。",
  },
  cleanDone: {
    en: "{removed} broken pages removed, {kept} kept.",
    id: "{removed} halaman rusak dibuang, {kept} halaman tersisa.",
    zh: "已移除 {removed} 个损坏页面，保留 {kept} 个。",
  },
  nothingToClean: {
    en: "No broken pages found.",
    id: "Tidak ada halaman rusak.",
    zh: "没有损坏的页面。",
  },
};

export const t = derived(lang, ($l) => (k: string): string => STR[k]?.[$l] ?? k);
export function tr(l: Lang, k: string): string {
  return STR[k]?.[l] ?? k;
}
