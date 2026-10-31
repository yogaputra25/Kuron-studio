// In-bubble editor (FE-5 canvas-redraw-split 3.3).
// Textarea HTML melayang di atas stage (Konva tak punya text-editing).
// Ketikan ditahan lokal; commit sekali via onEditTranslation. DOM ref
// (`editorEl`) tetap milik panel — hook minta fokus via `focusEditor`.
import type { BubbleBox, BubbleTranslation } from "../contracts/types";
import { computeTextLayout } from "./text/textLayout";
import { overlayFontFamily, textMeasure } from "./text/textMeasure";

export interface EditBox {
  left: number;
  top: number;
  width: number;
  minH: number;
  fontSize: number;
  fontFamily: string;
  lineHeight: number;
  dark: boolean;
}

export interface BubbleEditorDeps {
  getShowTranslation: () => boolean;
  getOnEditTranslation: () => ((i: number, value: string) => void) | undefined;
  getBubble: (i: number) => BubbleBox | undefined;
  findTranslation: (i: number) => BubbleTranslation | undefined;
  /** Kunci seleksi ke bubble baru + teruskan (anti-yank). */
  selectBubble: (i: number) => void;
  scale: () => number;
  getRelShape: (b: BubbleBox) => Array<[number, number]> | null;
  focusEditor: () => void;
  /** Nilai textarea (tetap milik panel karena `bind:value` butuh lvalue). */
  getEditingValue: () => string;
  setEditingValue: (v: string) => void;
}

export function useBubbleEditor(deps: BubbleEditorDeps) {
  let editingIdx = $state<number | null>(null);
  let editBox = $state<EditBox | null>(null);

  // Editor in-bubble TIDAK punya rumus sendiri: ukurannya dari
  // `computeTextLayout` yang sama dengan overlay, diskalakan untuk tampilan.
  function openEditor(i: number) {
    if (!deps.getShowTranslation() || !deps.getOnEditTranslation()) return;
    const b = deps.getBubble(i);
    const tr = deps.findTranslation(i);
    if (!b) return;
    // Kunci seleksi ke bubble baru DULU sebelum commit editor lama, supaya
    // commit(editTr) tak menarik manualSel balik ke bubble lama (yank).
    deps.selectBubble(i);
    commitEditor();
    const s = deps.scale();
    editingIdx = i;
    deps.setEditingValue(tr?.translated ?? "");
    // Layout yang sama dengan overlay — bukan rumus sendiri. Dihitung sekali.
    const lay = computeTextLayout(tr?.translated ?? "", b.w, b.h, textMeasure, { fontSize: tr?.fontSize, shape: deps.getRelShape(b) });
    editBox = {
      left: b.x * s,
      top: b.y * s,
      width: Math.max(60, b.w * s),
      minH: Math.max(24, b.h * s),
      fontSize: lay.fontSize * s,
      fontFamily: overlayFontFamily(),
      lineHeight: lay.lineHeight,
      dark: !(tr?.needsWhitePatch ?? false),
    };
    deps.focusEditor();
  }

  /** Commit dari luar (ganti tab): pola sama seperti deleteSelected. */
  function commitEditor() {
    if (editingIdx === null || !deps.getOnEditTranslation()) {
      editingIdx = null;
      return;
    }
    const i = editingIdx;
    const cur = deps.findTranslation(i)?.translated ?? "";
    editingIdx = null;
    editBox = null;
    // Nilai tak berubah → tutup saja, jangan tandai edited / antre save.
    if (cur !== deps.getEditingValue()) deps.getOnEditTranslation()!(i, deps.getEditingValue());
  }

  function cancelEditor() {
    editingIdx = null;
    editBox = null;
  }

  function onEditorKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      cancelEditor();
    } else if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      commitEditor();
    }
  }

  return {
    get editingIdx() { return editingIdx; },
    get editBox() { return editBox; },
    openEditor,
    commitEditor,
    cancelEditor,
    onEditorKey,
  };
}

export type BubbleEditor = ReturnType<typeof useBubbleEditor>;
