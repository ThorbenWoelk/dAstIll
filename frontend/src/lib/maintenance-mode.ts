const TRUE_VALUES = new Set(["1", "true", "yes", "on"]);

export function resolveMaintenanceMode(configuredValue?: string): boolean {
  const normalizedValue = configuredValue?.trim().toLowerCase();
  if (!normalizedValue) {
    return false;
  }

  return TRUE_VALUES.has(normalizedValue);
}
