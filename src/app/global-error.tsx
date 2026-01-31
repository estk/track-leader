"use client";

import { useEffect } from "react";

interface GlobalErrorProps {
  error: Error & { digest?: string };
  reset: () => void;
}

/**
 * Logs structured error data for production monitoring.
 * Output is JSON that can be parsed by log aggregators.
 */
function logStructuredError(error: Error & { digest?: string }, context: string) {
  const errorData = {
    type: "frontend_error",
    context,
    timestamp: new Date().toISOString(),
    url: typeof window !== "undefined" ? window.location.href : "unknown",
    userAgent: typeof navigator !== "undefined" ? navigator.userAgent : "unknown",
    error: {
      name: error.name,
      message: error.message,
      digest: error.digest,
      stack: error.stack,
    },
  };

  // Log as JSON for log aggregators
  console.error("[ERROR]", JSON.stringify(errorData));
}

/**
 * Global error boundary for catching errors in the root layout.
 * This is a fallback for when errors escape the normal error.tsx boundary.
 * Must include its own <html> and <body> tags since it replaces the root layout.
 */
export default function GlobalError({ error, reset }: GlobalErrorProps) {
  useEffect(() => {
    logStructuredError(error, "global_error_boundary");
  }, [error]);

  return (
    <html lang="en">
      <body>
        <div
          style={{
            minHeight: "100vh",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            padding: "1rem",
            fontFamily: "system-ui, -apple-system, sans-serif",
            backgroundColor: "#0a0a0a",
            color: "#fafafa",
          }}
        >
          <div
            style={{
              maxWidth: "28rem",
              width: "100%",
              textAlign: "center",
              padding: "2rem",
              backgroundColor: "#171717",
              borderRadius: "0.5rem",
              border: "1px solid #262626",
            }}
          >
            <div style={{ fontSize: "4rem", marginBottom: "1rem" }}>&#x26A0;&#xFE0F;</div>
            <h1 style={{ fontSize: "1.5rem", fontWeight: "bold", marginBottom: "1rem" }}>
              Something went wrong
            </h1>
            <p style={{ color: "#a3a3a3", marginBottom: "1.5rem" }}>
              A critical error occurred. Please try refreshing the page.
            </p>
            <button
              onClick={reset}
              style={{
                padding: "0.5rem 1rem",
                backgroundColor: "#fafafa",
                color: "#0a0a0a",
                border: "none",
                borderRadius: "0.375rem",
                cursor: "pointer",
                fontWeight: "500",
              }}
            >
              Try Again
            </button>
          </div>
        </div>
      </body>
    </html>
  );
}
