// @vitest-environment jsdom
// Bridge notify mentah (via contracts/log.call, bukan binding JS plugin):
// status/request/kirim satu panggilan per aksi, best-effort.
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  notificationStatus,
  notifyBatchDone,
  requestNotificationPermission,
  trySendNotification,
} from "./notify";

const { call } = vi.hoisted(() => ({
  call: vi.fn(),
}));

vi.mock("../contracts/log", () => ({
  call: (...a: unknown[]) => call(...a),
}));

vi.mock("./device", () => ({ isMobile: false }));

describe("notifyBatchDone", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    call.mockResolvedValue(true);
  });

  it("cek status dulu, kirim sekali saat granted", async () => {
    call.mockResolvedValueOnce(true).mockResolvedValueOnce(undefined);
    await notifyBatchDone("Batch selesai", "2 berhasil, 0 gagal");
    expect(call).toHaveBeenCalledWith("plugin:notification|is_permission_granted");
    expect(call).toHaveBeenCalledWith("plugin:notification|notify", {
      options: { title: "Batch selesai", body: "2 berhasil, 0 gagal" },
    });
  });

  it("diam tanpa kirim bila status bukan granted", async () => {
    call.mockResolvedValue(false);
    await expect(notifyBatchDone("T", "B")).resolves.toBeUndefined();
    expect(call).toHaveBeenCalledTimes(1);
  });

  it("diam bila invoke gagal (tanpa lempar)", async () => {
    call.mockRejectedValue(new Error("no plugin"));
    await expect(notifyBatchDone("T", "B")).resolves.toBeUndefined();
  });
});

describe("notificationStatus", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("granted saat command mengembalikan true", async () => {
    call.mockResolvedValue(true);
    await expect(notificationStatus()).resolves.toBe("granted");
  });

  it("not-granted saat command mengembalikan false", async () => {
    call.mockResolvedValue(false);
    await expect(notificationStatus()).resolves.toBe("not-granted");
  });

  it("unknown (bukan denied) saat invoke gagal", async () => {
    call.mockRejectedValue(new Error("denied by capability"));
    await expect(notificationStatus()).resolves.toBe("unknown");
  });
});

describe("requestNotificationPermission", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("granted bila OS menyetujui", async () => {
    call.mockResolvedValue("granted");
    await expect(requestNotificationPermission()).resolves.toBe("granted");
  });

  it("not-granted bila OS menolak (tanpa lempar)", async () => {
    call.mockResolvedValue("denied");
    await expect(requestNotificationPermission()).resolves.toBe("not-granted");
  });

  it("unknown saat invoke gagal (tanpa lempar)", async () => {
    call.mockRejectedValue(new Error("no plugin"));
    await expect(requestNotificationPermission()).resolves.toBe("unknown");
  });
});

describe("trySendNotification", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    call.mockResolvedValue(undefined);
  });

  it("null saat perintah terkirim tanpa error", async () => {
    await expect(trySendNotification("T", "B")).resolves.toBeNull();
    expect(call).toHaveBeenCalledWith("plugin:notification|notify", {
      options: { title: "T", body: "B" },
    });
  });

  it("mengembalikan pesan error (bukan melempar) saat kirim gagal", async () => {
    call.mockRejectedValue(new Error("OS refused delivery"));
    await expect(trySendNotification("T", "B")).resolves.toContain("OS refused delivery");
  });
});
