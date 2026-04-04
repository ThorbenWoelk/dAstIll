type TokenResolver = () => Promise<string | null>;

let resolveCurrentToken: TokenResolver = async () => null;

export function configureAuthTokenResolver(resolver: TokenResolver) {
  resolveCurrentToken = resolver;
}

export async function getCurrentAuthToken(): Promise<string | null> {
  return resolveCurrentToken();
}
