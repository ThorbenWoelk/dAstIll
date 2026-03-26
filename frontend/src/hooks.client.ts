import type { HandleClientError } from "@sveltejs/kit";

export const handleError: HandleClientError = ({ error, event }) => {
  const errorId = crypto.randomUUID();

  // Log client-side errors to console for now
  console.error(`[Client Error ${errorId}]`, {
    error,
    url: event.url.pathname,
  });

  return {
    message: "Whoops! Something went wrong in the browser.",
    errorId,
  };
};
