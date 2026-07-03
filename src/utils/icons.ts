import {
  LayoutDashboard,
  Package,
  HardDrive,
  Globe,
  Settings,
  FileText,
  Code,
  ScrollText,
  Database,
} from "@lucide/vue";

export const iconMap: Record<string, any> = {
  LayoutDashboard,
  Package,
  HardDrive,
  Database,
  Globe,
  Settings,
  FileText,
  Code,
  ScrollText,
};

export type IconName = keyof typeof iconMap;
