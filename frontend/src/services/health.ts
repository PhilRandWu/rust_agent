import { API_ENDPOINTS } from "@/constants/api";
import { getJson } from "@/services/http";

export interface HealthResponse {
  status: string;
}

export function fetchHealth(): Promise<HealthResponse> {
  return getJson<HealthResponse>(API_ENDPOINTS.health);
}
