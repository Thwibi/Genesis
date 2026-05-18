import { create } from "zustand";
import { api, PlatformDto, OtpDto } from "../services/api";

interface PlatformState {
  platforms: PlatformDto[];
  selectedOtp: OtpDto | null;
  loading: boolean;
  error: string | null;
  clipboardCountdown: number;
  clipboardActive: boolean;
  fetchPlatforms: () => Promise<void>;
  generateOtp: (sessionId: string, platformId: string) => Promise<OtpDto>;
  copyOtp: (sessionId: string, platformId: string, otp: string) => Promise<void>;
  startClipboardTimer: (timeout: number) => void;
  clearOtp: () => void;
}

export const usePlatformStore = create<PlatformState>((set) => ({
  platforms: [],
  selectedOtp: null,
  loading: false,
  error: null,
  clipboardCountdown: 0,
  clipboardActive: false,

  fetchPlatforms: async () => {
    set({ loading: true, error: null });
    try {
      const platforms = await api.listPlatforms();
      set({ platforms, loading: false });
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to fetch platforms";
      set({ error: message, loading: false });
    }
  },

  generateOtp: async (sessionId: string, platformId: string) => {
    const otp = await api.generateOtp({ session_id: sessionId, platform_id: platformId });
    set({ selectedOtp: otp });
    return otp;
  },

  copyOtp: async (sessionId: string, platformId: string, otp: string) => {
    await api.copyOtp(sessionId, platformId, otp);
  },

  startClipboardTimer: (timeout: number) => {
    set({ clipboardCountdown: timeout, clipboardActive: true });
    const interval = setInterval(() => {
      set((state) => {
        if (state.clipboardCountdown <= 1) {
          clearInterval(interval);
          return { clipboardCountdown: 0, clipboardActive: false, selectedOtp: null };
        }
        return { clipboardCountdown: state.clipboardCountdown - 1 };
      });
    }, 1000);
  },

  clearOtp: () => set({ selectedOtp: null, clipboardActive: false, clipboardCountdown: 0 }),
}));
