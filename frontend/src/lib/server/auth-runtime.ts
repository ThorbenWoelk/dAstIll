import { dev } from "$app/environment";
import { env } from "$env/dynamic/private";

export interface AuthRuntimeConfig {
  backendApiBase: string;
  backendProxyToken: string;
  backendIdentityAudience?: string;
}

const LOCAL_DEV_BACKEND_API_BASE = "http://localhost:3544";
const LOCAL_DEV_BACKEND_PROXY_TOKEN = "local-dev-backend-proxy-token";

function normalizeConfiguredValue(
  value: string | undefined,
): string | undefined {
  const normalized = value?.trim();
  return normalized ? normalized : undefined;
}

function requiredPrivateEnv(key: string, localDevFallback?: string): string {
  const configuredValue = normalizeConfiguredValue(env[key]);
  if (configuredValue) {
    return configuredValue;
  }

  if (dev && localDevFallback) {
    return localDevFallback;
  }

  throw new Error(`${key} must be set`);
}

function loadAuthRuntimeConfig(): AuthRuntimeConfig {
  const backendIdentityAudience = normalizeConfiguredValue(
    env.BACKEND_IDENTITY_AUDIENCE,
  );

  return {
    backendApiBase: requiredPrivateEnv(
      "BACKEND_API_BASE",
      LOCAL_DEV_BACKEND_API_BASE,
    ),
    backendProxyToken: requiredPrivateEnv(
      "BACKEND_PROXY_TOKEN",
      LOCAL_DEV_BACKEND_PROXY_TOKEN,
    ),
    ...(backendIdentityAudience ? { backendIdentityAudience } : {}),
  };
}

export function getAuthRuntimeConfig(): AuthRuntimeConfig {
  return loadAuthRuntimeConfig();
}
