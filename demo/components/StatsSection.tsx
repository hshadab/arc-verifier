export default function StatsSection() {
  const stats = [
    {
      value: '~1M',
      label: 'Gas Per Verification',
      subtext: 'Only ~$0.02 on Arc',
      icon: '⛽',
    },
    {
      value: '~2-5s',
      label: 'Proof Generation',
      subtext: 'After initialization',
      icon: '⚡',
    },
    {
      value: '~128B',
      label: 'Proof Size',
      subtext: 'Compact Groth16',
      icon: '📦',
    },
    {
      value: '3-in-1',
      label: 'Compliance Checks',
      subtext: 'All proven together',
      icon: '✅',
    },
  ]

  return (
    <div className="py-16 px-4 sm:px-6 lg:px-8 bg-arc-darker/50">
      <div className="max-w-7xl mx-auto">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
          {stats.map((stat, idx) => (
            <div key={idx} className="glass-effect p-6 rounded-xl text-center hover:glow-effect transition-all">
              <div className="text-4xl mb-3">{stat.icon}</div>
              <div className="text-3xl font-bold text-arc-primary mb-2">{stat.value}</div>
              <div className="text-sm font-semibold text-white mb-1">{stat.label}</div>
              <div className="text-xs text-gray-400">{stat.subtext}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
