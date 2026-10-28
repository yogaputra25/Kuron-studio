// Metrik baseline preview (grid-preview-perf 1.1): waktu round-trip per
// halaman + ukuran payload IPC + waktu thumbnail pertama. Murni — direct-import.
export interface PreviewSample {
  ms: number;
  bytes: number;
}

export interface PreviewSummary {
  n: number;
  avgMs: number;
  maxMs: number;
  totalBytes: number;
  timeToFirstMs: number;
}

/** Ringkas sampel; `firstMs` = waktu dari mulai antrean sampai thumb pertama. */
export function summarizePreviews(samples: PreviewSample[], firstMs: number): PreviewSummary {
  let total = 0;
  let max = 0;
  let bytes = 0;
  for (const s of samples) {
    total += s.ms;
    if (s.ms > max) max = s.ms;
    bytes += s.bytes;
  }
  return {
    n: samples.length,
    avgMs: samples.length === 0 ? 0 : Math.round((total / samples.length) * 10) / 10,
    maxMs: Math.round(max * 10) / 10,
    totalBytes: bytes,
    timeToFirstMs: Math.round(firstMs * 10) / 10,
  };
}
