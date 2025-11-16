'use client'

import Header from '@/components/Header'
import Hero from '@/components/Hero'
import StatsSection from '@/components/StatsSection'
import ContractsSection from '@/components/ContractsSection'
import TwoPhaseDemo from '@/components/TwoPhaseDemo'
import NovaProofDemo from '@/components/NovaProofDemo'
import Footer from '@/components/Footer'

export default function Home() {
  return (
    <main className="min-h-screen">
      <Header />
      <Hero />

      {/* Nova Recursive Proof Demo */}
      <NovaProofDemo />

      <StatsSection />

      {/* Groth16 Proof Generation and Verification */}
      <TwoPhaseDemo />

      <ContractsSection />
      <Footer />
    </main>
  )
}
