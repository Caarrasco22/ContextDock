import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "ContextDock",
  description: "A lightweight workspace for persistent project context",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-full antialiased bg-zinc-950 text-zinc-100">
        {children}
      </body>
    </html>
  );
}