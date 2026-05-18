import { create } from "zustand";
import { api, SessionDto } from "../services/api";

interface AuthState {
  session: SessionDto | null;
  loading: boolean;
  error: string | null;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  session: null,
  loading: false,
  error: null,

  login: async (username: string, password: string) => {
    set({ loading: true, error: null });
    try {
      const session = await api.login(username, password);
      set({ session, loading: false });
    } catch (err) {
      const message = err instanceof Error ? err.message : "Login failed";
      set({ error: message, loading: false });
      throw err;
    }
  },

  logout: async () => {
    const { session } = get();
    if (session) {
      try {
        await api.logout(session.id);
      } catch {
        // Proceed with local logout even if remote fails
      }
    }
    set({ session: null, error: null });
  },

  clearError: () => set({ error: null }),
}));
