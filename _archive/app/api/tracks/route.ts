import { NextRequest, NextResponse } from 'next/server'
import { db } from '@/lib/database'
import { parseGPX } from '@/lib/gpx-parser'

export async function GET() {
  try {
    const tracks = await db.getAllTracks()
    return NextResponse.json(tracks)
  } catch (error) {
    return NextResponse.json({ error: 'Failed to fetch tracks' }, { status: 500 })
  }
}

export async function POST(request: NextRequest) {
  try {
    const formData = await request.formData()
    const file = formData.get('file') as File

    if (!file) {
      return NextResponse.json({ error: 'No file provided' }, { status: 400 })
    }

    if (!file.name.toLowerCase().endsWith('.gpx')) {
      return NextResponse.json({ error: 'Only GPX files are supported' }, { status: 400 })
    }

    const content = await file.text()
    const parsedTrack = await parseGPX(content)

    const trackData = {
      name: parsedTrack.name,
      filename: file.name,
      uploadDate: new Date().toISOString(),
      distance: parsedTrack.distance,
      duration: parsedTrack.duration,
      elevationGain: parsedTrack.elevationGain,
      maxSpeed: parsedTrack.maxSpeed,
      avgSpeed: parsedTrack.avgSpeed,
      coordinates: JSON.stringify(parsedTrack.points)
    }

    const trackId = await db.insertTrack(trackData)

    return NextResponse.json({ id: trackId, ...trackData })
  } catch (error) {
    console.error('Error processing track:', error)
    return NextResponse.json({ error: 'Failed to process track' }, { status: 500 })
  }
}
