import sqlite3 from 'sqlite3'
import { promisify } from 'util'
import path from 'path'

export interface Track {
  id?: number
  name: string
  filename: string
  uploadDate: string
  distance: number
  duration: number
  elevationGain: number
  maxSpeed: number
  avgSpeed: number
  coordinates: string // JSON string of coordinates
}

class Database {
  private db: sqlite3.Database

  constructor() {
    const dbPath = path.join(process.cwd(), 'tracks.db')
    this.db = new sqlite3.Database(dbPath)
    this.initializeDatabase()
  }

  private async initializeDatabase() {
    const run = promisify(this.db.run.bind(this.db))

    await run(`
      CREATE TABLE IF NOT EXISTS tracks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        filename TEXT NOT NULL,
        uploadDate TEXT NOT NULL,
        distance REAL NOT NULL,
        duration INTEGER NOT NULL,
        elevationGain REAL NOT NULL,
        maxSpeed REAL NOT NULL,
        avgSpeed REAL NOT NULL,
        coordinates TEXT NOT NULL
      )
    `)
  }

  async insertTrack(track: Omit<Track, 'id'>): Promise<number> {
    const run = promisify(this.db.run.bind(this.db))
    const result = await run(
      `INSERT INTO tracks (name, filename, uploadDate, distance, duration, elevationGain, maxSpeed, avgSpeed, coordinates)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
      [track.name, track.filename, track.uploadDate, track.distance, track.duration,
       track.elevationGain, track.maxSpeed, track.avgSpeed, track.coordinates]
    )
    return (result as any).lastID
  }

  async getAllTracks(): Promise<Track[]> {
    const all = promisify(this.db.all.bind(this.db))
    return await all('SELECT * FROM tracks ORDER BY uploadDate DESC')
  }

  async getTrackById(id: number): Promise<Track | null> {
    const get = promisify(this.db.get.bind(this.db))
    const track = await get('SELECT * FROM tracks WHERE id = ?', [id])
    return track || null
  }
}

export const db = new Database()
