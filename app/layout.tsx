import type { Metadata } from "next";
import type { ReactNode } from "react";
import "./globals.css";

export const metadata: Metadata = {
  title: "Lume",
  description: "Desktop screen dimmer",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  return (
    <html lang="en" className="h-full bg-transparent">
      <body className="m-0 min-h-full bg-transparent">{children}</body>
    </html>
  );
}
