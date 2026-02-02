"use client";

import { ReactNode } from "react";
import { AuthProvider } from "@/lib/auth-context";
import { SidebarProvider } from "@/components/navigation";

export function Providers({ children }: { children: ReactNode }) {
  return (
    <AuthProvider>
      <SidebarProvider>{children}</SidebarProvider>
    </AuthProvider>
  );
}
