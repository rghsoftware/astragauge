import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { SensorReading } from './types';

const HISTORY_CAP = 60;

// Latest reading per sensor id.
let readings = $state<Record<string, SensorReading>>({});
// Bounded history of recent values per sensor id (capped at HISTORY_CAP).
let historyMap = $state<Record<string, number[]>>({});

let started = false;

function applyBatch(batch: SensorReading[]): void {
  for (const reading of batch) {
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
  started = true;

  try {
    const snapshot = await invoke<SensorReading[]>('get_sensor_snapshot');
    applyBatch(snapshot);

    await listen<SensorReading[]>('sensors-update', (event) => {
      applyBatch(event.payload);
    });
  } catch (err) {
    // Non-Tauri (e.g. browser) context or transport failure: log and continue
    // so the UI still renders with whatever data is available.
    console.warn('sensors.start() failed; continuing without live data', err);
  }
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
  getReading,
  getValue,
  getHistory,
  allReadings,
};
