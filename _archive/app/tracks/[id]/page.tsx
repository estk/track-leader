'use client'

import { useEffect, useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import TrackDetail from '@/components/TrackDetail'
import { Track } from '@/lib/database'
import { TrackPoint } from '@/lib/gpx-parser'

export default function TrackDetailPage() {
  const params = useParams()
  const router = useRouter()
  const [track, setTrack] = useState<(Track & { coordinates: TrackPoint[] }) | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (params.id) {
      fetchTrack(params.id as string)
    }
  }, [params.id])

  const fetchTrack = async (trackId: string) => {
    try {
      const response = await fetch(`/api/tracks/${trackId}`)
      if (!response.ok) {
        throw new Error('Track not found')
      }
      const trackData = await response.json()
      setTrack(trackData)
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to load track')
    } finally {
      setLoading(false)
    }
  }

  const handleBack = () => {
    router.push('/')
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-96">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        <span className="ml-2">Loading track...</span>
      </div>
    )
  }

  if (error) {
    return (
      <div className="bg-white rounded-lg shadow-md p-6 text-center">
        <p className="text-red-600 mb-4">{error}</p>
        <button
          onClick={handleBack}
          className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors"
        >
          Back to Tracks
        </button>
      </div>
    )
  }

  if (!track) {
    return (
      <div className="bg-white rounded-lg shadow-md p-6 text-center">
        <p className="text-gray-600">Track not found</p>
        <button
          onClick={handleBack}
          className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors mt-4"
        >
          Back to Tracks
        </button>
      </div>
    )
  }

  return <TrackDetail track={track} onBack={handleBack} />
}
