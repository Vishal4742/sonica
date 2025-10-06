import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';
import { RecognitionResult } from './audioProcessor';

interface RecognitionState {
  isRecording: boolean;
  isProcessing: boolean;
  currentResult: RecognitionResult | null;
  recognitionHistory: RecognitionResult[];
  error: string | null;
  
  // Actions
  startRecording: () => void;
  stopRecording: () => void;
  setProcessing: (processing: boolean) => void;
  setResult: (result: RecognitionResult | null) => void;
  addToHistory: (result: RecognitionResult) => void;
  setError: (error: string | null) => void;
  clearHistory: () => void;
}

export const useRecognitionStore = create<RecognitionState>()(
  devtools(
    persist(
      (set, get) => ({
        isRecording: false,
        isProcessing: false,
        currentResult: null,
        recognitionHistory: [],
        error: null,

        startRecording: () => set({ isRecording: true, error: null }),
        stopRecording: () => set({ isRecording: false }),
        setProcessing: (processing) => set({ isProcessing: processing }),
        setResult: (result) => set({ currentResult: result }),
        addToHistory: (result) => set((state) => ({
          recognitionHistory: [result, ...state.recognitionHistory.slice(0, 49)] // Keep last 50
        })),
        setError: (error) => set({ error }),
        clearHistory: () => set({ recognitionHistory: [] }),
      }),
      {
        name: 'sonica-recognition-storage',
        partialize: (state) => ({
          recognitionHistory: state.recognitionHistory,
        }),
      }
    )
  )
);

// Search state
interface SearchState {
  query: string;
  results: any[];
  isSearching: boolean;
  filters: {
    genre: string[];
    language: string[];
    year: { min: number; max: number };
    duration: { min: number; max: number };
  };
  
  setQuery: (query: string) => void;
  setResults: (results: any[]) => void;
  setSearching: (searching: boolean) => void;
  updateFilters: (filters: Partial<SearchState['filters']>) => void;
  clearFilters: () => void;
}

export const useSearchStore = create<SearchState>()(
  devtools((set) => ({
    query: '',
    results: [],
    isSearching: false,
    filters: {
      genre: [],
      language: [],
      year: { min: 1900, max: new Date().getFullYear() },
      duration: { min: 0, max: 600 },
    },

    setQuery: (query) => set({ query }),
    setResults: (results) => set({ results }),
    setSearching: (searching) => set({ isSearching: searching }),
    updateFilters: (newFilters) => set((state) => ({
      filters: { ...state.filters, ...newFilters }
    })),
    clearFilters: () => set({
      filters: {
        genre: [],
        language: [],
        year: { min: 1900, max: new Date().getFullYear() },
        duration: { min: 0, max: 600 },
      }
    }),
  }))
);

// User preferences
interface UserPreferences {
  theme: 'light' | 'dark' | 'auto';
  language: string;
  autoRecord: boolean;
  notifications: boolean;
  analytics: boolean;
  
  updatePreferences: (prefs: Partial<UserPreferences>) => void;
}

export const useUserPreferences = create<UserPreferences>()(
  devtools(
    persist(
      (set) => ({
        theme: 'auto',
        language: 'en',
        autoRecord: false,
        notifications: true,
        analytics: true,

        updatePreferences: (prefs) => set(prefs),
      }),
      {
        name: 'sonica-user-preferences',
      }
    )
  )
);
