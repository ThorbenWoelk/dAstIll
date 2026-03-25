export function isAnonymousChatQuotaError(message: string): boolean {
  return message.includes("Anonymous chat quota exceeded");
}
