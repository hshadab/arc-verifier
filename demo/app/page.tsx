'use client'

import { useState } from 'react'
import Header from '@/components/Header'
import Hero from '@/components/Hero'
import DemoSection from '@/components/DemoSection'
import StatsSection from '@/components/StatsSection'
import ContractsSection from '@/components/ContractsSection'
import Footer from '@/components/Footer'

export default function Home() {
  const [proofStatus, setProofStatus] = useState<'idle' | 'generating' | 'verifying' | 'verified'>('idle')

  return (
    <main className="min-h-screen">
      <Header />
      <Hero />
      <StatsSection />
      <DemoSection proofStatus={proofStatus} setProofStatus={setProofStatus} />
      <ContractsSection />
      <Footer />
    </main>
  )
}
