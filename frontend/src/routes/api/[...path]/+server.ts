import type { RequestHandler } from "./$types";

import { getAuthRuntimeConfig } from "$lib/server/auth";

const HOP_BY_HOP_HEADERS = new Set([
  "connection",
  "content-length",
  "keep-alive",
  "proxy-authenticate",
  "proxy-authorization",
  "te",
  "trailer",
  "transfer-encoding",
  "upgrade",
]);

function buildBackendUrl(baseUrl: string, path: string, search: string): URL {
  const normalizedBaseUrl = baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`;
  const targetUrl = new URL(`api/${path}`, normalizedBaseUrl);
  targetUrl.search = search;
  return targetUrl;
}

function copyProxyRequestHeaders(sourceHeaders: Headers): Headers {
  const headers = new Headers();
  for (const [name, value] of sourceHeaders.entries()) {
    const lowercaseName = name.toLowerCase();
    if (
      HOP_BY_HOP_HEADERS.has(lowercaseName) ||
      lowercaseName === "authorization" ||
      lowercaseName === "cookie" ||
      lowercaseName === "host"
    ) {
      continue;
    }

    headers.set(name, value);
  }

  return headers;
}

function copyProxyResponseHeaders(sourceHeaders: Headers): Headers {
  const headers = new Headers();
  for (const [name, value] of sourceHeaders.entries()) {
    if (HOP_BY_HOP_HEADERS.has(name.toLowerCase())) {
      continue;
    }

    headers.set(name, value);
  }

  return headers;
}

async function fetchIdentityToken(
  audience: string | undefined,
): Promise<string | null> {
  if (!audience) {
    return null;
  }

  const tokenUrl = new URL(
    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity",
  );
  tokenUrl.searchParams.set("audience", audience);
  tokenUrl.searchParams.set("format", "full");

  const response = await fetch(tokenUrl, {
    headers: {
      "Metadata-Flavor": "Google",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to mint backend identity token (${response.status})`,
    );
  }

  const token = (await response.text()).trim();
  return token || null;
}

const proxyRequest: RequestHandler = async (event) => {
  const authConfig = getAuthRuntimeConfig();
  const targetUrl = buildBackendUrl(
    authConfig.backendApiBase,
    event.params.path ?? "",
    event.url.search,
  );

  const headers = copyProxyRequestHeaders(event.request.headers);
  headers.set("x-dastill-proxy-auth", authConfig.backendProxyToken);
  headers.set("x-dastill-role", event.locals.session?.role ?? "operator");

  try {
    headers.set("x-dastill-client-ip", event.getClientAddress());
  } catch {
    headers.set("x-dastill-client-ip", "unknown");
  }

  try {
    const identityToken = await fetchIdentityToken(
      authConfig.backendIdentityAudience,
    );
    if (identityToken) {
      headers.set("Authorization", `Bearer ${identityToken}`);
    }
  } catch (error) {
    return new Response((error as Error).message, {
      status: 502,
    });
  }

  const requestHasBody = !["GET", "HEAD"].includes(event.request.method);
  const body = requestHasBody
    ? Buffer.from(await event.request.arrayBuffer())
    : undefined;

  let backendResponse: Response;
  try {
    backendResponse = await fetch(targetUrl, {
      method: event.request.method,
      headers,
      body,
      redirect: "manual",
    });
  } catch (error) {
    return new Response((error as Error).message || "Backend proxy failed", {
      status: 502,
    });
  }

  return new Response(backendResponse.body, {
    status: backendResponse.status,
    headers: copyProxyResponseHeaders(backendResponse.headers),
  });
};

export const GET = proxyRequest;
export const HEAD = proxyRequest;
export const POST = proxyRequest;
export const PUT = proxyRequest;
export const DELETE = proxyRequest;
export const OPTIONS = proxyRequest;
