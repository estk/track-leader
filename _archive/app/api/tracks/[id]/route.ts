import { NextRequest, NextResponse } from 'next/server'
import { db } from '@/lib/database'

export async function GET(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  try {
    const trackId = parseInt(params.id)
    if (isNaN(trackId)) {
      return NextResponse.json({ error: 'Invalid track ID' }, { status: 400 })
    }

    const track = await db.getTrackById(trackId)

    if (!track) {
      return NextResponse.json({ error: 'Track not found' }, { status: 404 })
    }

    return NextResponse.json({
      ...track,
      coordinates: JSON.parse(track.coordinates)
    })
  } catch (error) {
    return NextResponse.json({ error: 'Failed to fetch track' }, { status: 500 })
  }
}
