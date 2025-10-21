import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Cesium Globe - Next.js",
  description: "3D Globe visualization using Cesium.js",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        {children}
      </body>
    </html>
  );
}
