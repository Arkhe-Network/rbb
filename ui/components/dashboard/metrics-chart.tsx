'use client';

import { AnimatedCard } from '@/components/cult/animated-card';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

const dummyData = [
  { time: '10:00', value: 400 },
  { time: '10:05', value: 300 },
  { time: '10:10', value: 550 },
  { time: '10:15', value: 450 },
  { time: '10:20', value: 600 },
];

export function MetricsChart({ className }: { className?: string }) {
  return (
    <AnimatedCard className={`h-96 ${className || ''}`}>
      <h3 className="text-zinc-400 text-sm mb-4">Throughput (tokens/s)</h3>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={dummyData}>
          <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
          <XAxis dataKey="time" stroke="#a1a1aa" />
          <YAxis stroke="#a1a1aa" />
          <Tooltip contentStyle={{ backgroundColor: '#18181b', border: 'none' }} />
          <Line type="monotone" dataKey="value" stroke="#f59e0b" strokeWidth={2} dot={false} />
        </LineChart>
      </ResponsiveContainer>
    </AnimatedCard>
  );
}
