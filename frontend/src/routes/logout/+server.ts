import { redirect } from "@sveltejs/kit";

import type { RequestHandler } from "./$types";
import { clearSessionCookie } from "$lib/server/auth";

const logout: RequestHandler = ({ cookies }) => {
  clearSessionCookie(cookies);
  throw redirect(303, "/login");
};

export const GET = logout;
export const POST = logout;
