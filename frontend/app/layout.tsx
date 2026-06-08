import type {Metadata} from 'next';
import { Hanken_Grotesk, JetBrains_Mono } from 'next/font/google';
import './globals.css';

const hanken = Hanken_Grotesk({
  subsets: ['latin'],
  variable: '--font-sans',
});

const jetbrains = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-mono',
});

export const metadata: Metadata = {
  title: 'Axeane Automation Sync',
  description: 'Data sync and automation tool',
};

export default function RootLayout({children}: {children: React.ReactNode}) {
  return (
    <html lang="en" className={`${hanken.variable} ${jetbrains.variable} dark`}>
      <body suppressHydrationWarning className="bg-background text-on-background font-body-md overflow-hidden">{children}</body>
    </html>
  );
}
