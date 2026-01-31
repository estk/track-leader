"use client";

import { useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface ErrorPageProps {
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

export default function ErrorPage({ error, reset }: ErrorPageProps) {
  useEffect(() => {
    logStructuredError(error, "error_boundary");
  }, [error]);

  return (
    <div className="min-h-[60vh] flex items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 text-6xl">&#x26A0;&#xFE0F;</div>
          <CardTitle className="text-2xl">Something went wrong</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4 text-center">
          <p className="text-muted-foreground">
            We encountered an unexpected error. Please try again or contact
            support if the problem persists.
          </p>
          {process.env.NODE_ENV === "development" && (
            <details className="text-left text-sm">
              <summary className="cursor-pointer text-muted-foreground">
                Error details
              </summary>
              <pre className="mt-2 p-2 bg-muted rounded text-xs overflow-auto">
                {error.message}
              </pre>
            </details>
          )}
          <div className="flex gap-2 justify-center pt-4">
            <Button variant="outline" onClick={() => window.history.back()}>
              Go Back
            </Button>
            <Button onClick={reset}>Try Again</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
