export function parseUrl(raw: string): URL | null {
  try {
    return new URL(raw);
  } catch {
    return null;
  }
}

export type UrlParts = {
  domain: string;
  rest: string;
};

export function displayUrl(raw: string): UrlParts {
  const parsed = parseUrl(raw);
  if (parsed === null) return { domain: raw, rest: "" };
  const domain = parsed.hostname || parsed.protocol.replace(":", "");
  const rest = `${parsed.pathname}${parsed.search}${parsed.hash}`;
  return { domain, rest };
}

export type UrlSegments = {
  scheme: string;
  subdomain: string;
  primary: string;
  tail: string;
};

export function describeUrl(raw: string): UrlSegments | null {
  const parsed = parseUrl(raw);
  if (parsed === null) return null;

  const labels = parsed.hostname.split(".").filter(Boolean);
  let subdomain = "";
  let primary: string;
  if (labels.length >= 2) {
    primary = labels.slice(-2).join(".");
    subdomain = labels.slice(0, -2).join(".");
  } else {
    primary = labels[0] ?? parsed.hostname;
  }
  if (parsed.port) primary += `:${parsed.port}`;

  const scheme = parsed.protocol;
  const tail = `${parsed.pathname}${parsed.search}${parsed.hash}`;
  return { scheme, subdomain, primary, tail };
}

const TRACKING_PREFIXES = ["utm_", "ref_", "ga_", "_ga", "mc_"];

const TRACKING_EXACT = new Set([
  "fbclid",
  "gclid",
  "dclid",
  "msclkid",
  "yclid",
  "igshid",
  "spm",
  "cmpid",
]);

export function isTrackingParam(name: string): boolean {
  const lower = name.toLowerCase();
  if (TRACKING_EXACT.has(lower)) return true;
  return TRACKING_PREFIXES.some((prefix) => lower.startsWith(prefix));
}

export function hasTrackingParams(raw: string): boolean {
  const parsed = parseUrl(raw);
  if (parsed === null) return false;
  const search = parsed.search.replace(/^\?/, "");
  for (const part of search.split("&")) {
    if (!part) continue;
    const [name] = part.split("=");
    if (name && isTrackingParam(name)) return true;
  }
  return false;
}

export function isSanitized(raw: string, clean: string): boolean {
  if (raw === clean) return false;
  const rawUrl = parseUrl(raw);
  const cleanUrl = parseUrl(clean);
  if (rawUrl === null || cleanUrl === null) return false;
  return rawUrl.search !== cleanUrl.search && hasTrackingParams(raw);
}