// AI Usage types
export interface PeriodUsage {
  utilization: number;
  resets_at: string;
}

export interface ExtraUsageData {
  is_enabled: boolean;
  monthly_limit: number | null;
  used_credits: number | null;
  utilization: number | null;
}

export interface UsageData {
  five_hour: PeriodUsage;
  seven_day: PeriodUsage;
  extra_usage: ExtraUsageData;
}

export interface CodexCredits {
  remaining: number;
  has_credits: boolean;
}

export interface CodexUsageData {
  primary: PeriodUsage;
  secondary: PeriodUsage | null;
  credits: CodexCredits | null;
}

export interface AllUsage {
  claude: UsageData | null;
  codex: CodexUsageData | null;
}

// System metrics types
export interface CpuMetrics {
  overall: number;
  per_core: number[];
}

export interface RamMetrics {
  used_gb: number;
  total_gb: number;
}

export interface DiskMetrics {
  used_gb: number;
  total_gb: number;
  read_speed: number;
  write_speed: number;
}

export interface NetMetrics {
  download_speed: number;
  upload_speed: number;
}

export interface BatteryMetrics {
  percent: number;
  health_percent: number;
  cycle_count: number;
  charging: boolean;
  time_to_full: number | null;
  time_to_empty: number | null;
}

export interface TempSensor {
  name: string;
  value: number;
}

export interface FanInfo {
  name: string;
  rpm: number;
  min: number;
  max: number;
}

export interface BtDevice {
  name: string;
  connected: boolean;
  battery: number | null;
}

export interface SystemMetrics {
  cpu: CpuMetrics;
  ram: RamMetrics;
  disk: DiskMetrics;
  network: NetMetrics;
  battery: BatteryMetrics | null;
  temps: TempSensor[];
  fans: FanInfo[];
  bluetooth: BtDevice[];
}

// Settings types
export interface TraySettings {
  items: string[];
  separator: string;
  show_labels: boolean;
  show_units: boolean;
}

export interface PollingSettings {
  ai_interval_sec: number;
  system_interval_sec: number;
}

export interface SectionConfig {
  visible: boolean;
  order: number;
}

export interface PopupSettings {
  sections: Record<string, SectionConfig>;
}

export interface UserSettings {
  schema_version: number;
  theme: string;
  tray: TraySettings;
  polling: PollingSettings;
  popup: PopupSettings;
}
