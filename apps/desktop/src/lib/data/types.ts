export interface SensorReading {
  id: string;
  name: string;
  category: string;
  unit: string;
  value: number | null;
  timestampMs: number;
}
