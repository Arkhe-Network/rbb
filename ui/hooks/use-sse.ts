import { useEffect, useRef, useCallback } from 'react';
import { useCathedralStore } from '@/lib/store';

export function useSSE(url: string, onMessage: (data: any) => void) {
  const { setSseConnected } = useCathedralStore();
  useEffect(() => {
    const eventSource = new EventSource(url);
    eventSource.onopen = () => setSseConnected(true);
    eventSource.onmessage = (event) => {
      try { onMessage(JSON.parse(event.data)); } catch (e) {}
    };
    eventSource.onerror = () => setSseConnected(false);
    return () => eventSource.close();
  }, [url, onMessage, setSseConnected]);
}
