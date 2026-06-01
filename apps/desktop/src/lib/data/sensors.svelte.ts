import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { SensorReading } from './types';

const HISTORY_CAP = 60;

let readings = $state<Record<string, SensorReading>>({});
let historyMap = $state<Record<string, number[]>>({});
let initError = $state<string | null>(null);

let started = false;
let unlisten: (() => void) | null = null;

function isValidReading(r: SensorReading): boolean {
  return (
    typeof r.id === 'string' &&
    r.id.length > 0 &&
    (r.value === null ||
      (typeof r.value === 'number' && Number.isFinite(r.value))) &&
    typeof r.timestampMs === 'number' &&
    r.timestampMs >= 0
  );
}

function applyBatch(batch: SensorReading[]): void {
  for (const reading of batch) {
    if (!isValidReading(reading)) {
      console.warn('[sensors] dropping malformed reading:', reading);
      continue;
    }
    readings[reading.id] = reading;
    if (reading.value !== null) {
      const prev = historyMap[reading.id] ?? [];
      historyMap[reading.id] = [...prev, reading.value].slice(-HISTORY_CAP);
    }
  }
}

async function start(): Promise<void> {
  if (started) {
    return;
  }

  try {
    const snapshot = await invoke<SensorReading[]>('get_sensor_snapshot');
    applyBatch(snapshot);

    unlisten = await listen<SensorReading[]>('sensors-update', (event) => {
      applyBatch(event.payload);
    });

    started = true;
  } catch (err) {
    initError = err instanceof Error ? err.message : String(err);
    console.error('[sensors] Failed to initialize live data pipeline:', err);
  }
}

function stop(): void {
  unlisten?.();
  unlisten = null;
  started = false;
}

function getReading(id: string): SensorReading | undefined {
  return readings[id];
}

function getValue(id: string): number | null {
  return readings[id]?.value ?? null;
}

function getHistory(id: string): number[] {
  return historyMap[id] ?? [];
}

function allReadings(): SensorReading[] {
  return Object.values(readings);
}

export const sensors = {
  start,
  stop,
  getReading,
  getValue,
  getHistory,
  allReadings,
  get initError() {
    return initError;
  },
};
