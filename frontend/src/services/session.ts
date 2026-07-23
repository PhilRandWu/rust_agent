import { API_ENDPOINTS } from "@/constants/api";
import { getJson, HttpError } from "@/services/http";
import type { GetVersionResponse, ListVersionsResponse } from "@/types/session";

export function listVersions(projectId: string): Promise<ListVersionsResponse> {
  return getJson<ListVersionsResponse>(
    API_ENDPOINTS.sessionVersions(projectId),
  );
}

export function getVersion(
  projectId: string,
  version: number,
): Promise<GetVersionResponse> {
  return getJson<GetVersionResponse>(
    API_ENDPOINTS.sessionVersion(projectId, version),
  );
}

export async function deleteSession(projectId: string): Promise<void> {
  const res = await fetch(API_ENDPOINTS.session(projectId), {
    method: "DELETE",
  });
  if (!res.ok) {
    throw new HttpError(res.status, `DELETE session -> ${res.status}`);
  }
}
