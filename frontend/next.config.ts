import type { NextConfig } from "next";

const BACKEND_URL =
  process.env.NEXT_PUBLIC_BACKEND_URL ?? "http://localhost:7001";

const nextConfig: NextConfig = {
  async rewrites() {
    return [
      // /health 后端不带 /api 前缀，单独映射
      {
        source: "/api/health",
        destination: `${BACKEND_URL}/health`,
      },
      // 其余 /api/* 原样透传（后端路由含 /api 前缀）
      {
        source: "/api/:path*",
        destination: `${BACKEND_URL}/api/:path*`,
      },
    ];
  },
};

export default nextConfig;
