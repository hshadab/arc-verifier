export default function StatsSection() {
  const stats = [
    {
      value: '795K',
      label: 'Gas Per Verification',
      subtext: 'Only ~$0.016 on Arc',
      icon: '⛽',
    },
    {
      value: '22s',
      label: 'Proof Generation',
      subtext: 'Off-chain proving time',
      icon: '⚡',
    },
    {
      value: '900B',
      label: 'Proof Size',
      subtext: 'Constant, compact',
      icon: '📦',
    },
    {
      value: '1,427x',
      label: 'Cost Savings',
      subtext: 'vs Traditional Audits',
      icon: '💰',
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
