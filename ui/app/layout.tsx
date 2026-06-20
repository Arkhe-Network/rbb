import './globals.css';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Cathedral ARKHE',
  description: 'Cathedral ARKHE - Unified Operations',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="bg-zinc-950 text-zinc-100 min-h-screen font-sans antialiased">
        {children}
      </body>
    </html>
  );
}
