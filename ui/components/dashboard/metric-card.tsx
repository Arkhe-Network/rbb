import { AnimatedCard } from '@/components/cult/animated-card';

export function MetricCard({ title, value }: { title: string; value: string | number }) {
  return (
    <AnimatedCard className="flex flex-col gap-2">
      <div className="text-zinc-400 text-sm">{title}</div>
      <div className="text-2xl font-bold">{value}</div>
    </AnimatedCard>
  );
}
