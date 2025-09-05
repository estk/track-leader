import xml2js from "xml2js";

export interface TrackPoint {
  lat: number;
  lon: number;
  ele?: number;
  time?: string;
}
export function as_latlon(x: TrackPoint): [number, number] {
  return [x.lat, x.lon];
}

export interface ParsedTrack {
  name: string;
  points: TrackPoint[];
  distance: number;
  duration: number;
  elevationGain: number;
  maxSpeed: number;
  avgSpeed: number;
}

function calculateDistance(lat1: number, lon1: number, lat2: number, lon2: number): number {
  const R = 6371000; // Earth's radius in meters
  const dLat = ((lat2 - lat1) * Math.PI) / 180;
  const dLon = ((lon2 - lon1) * Math.PI) / 180;
  const a =
    Math.sin(dLat / 2) * Math.sin(dLat / 2) +
    Math.cos((lat1 * Math.PI) / 180) * Math.cos((lat2 * Math.PI) / 180) * Math.sin(dLon / 2) * Math.sin(dLon / 2);
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
  return R * c;
}

function calculateSpeed(distance: number, timeDiff: number): number {
  return timeDiff > 0 ? (distance / timeDiff) * 3.6 : 0; // km/h
}

export async function parseGPX(gpxContent: string): Promise<ParsedTrack> {
  const parser = new xml2js.Parser();
  const result = await parser.parseStringPromise(gpxContent);

  const gpx = result.gpx;
  const track = gpx.trk[0];
  const trackName = track.name?.[0] || "Unnamed Track";
  const segments = track.trkseg;

  const points: TrackPoint[] = [];

  for (const segment of segments) {
    for (const point of segment.trkpt) {
      points.push({
        lat: parseFloat(point.$.lat),
        lon: parseFloat(point.$.lon),
        ele: point.ele?.[0] ? parseFloat(point.ele[0]) : undefined,
        time: point.time?.[0],
      });
    }
  }

  let totalDistance = 0;
  let elevationGain = 0;
  let maxSpeed = 0;
  const speeds: number[] = [];

  for (let i = 1; i < points.length; i++) {
    const prev = points[i - 1];
    const curr = points[i];

    const distance = calculateDistance(prev.lat, prev.lon, curr.lat, curr.lon);
    totalDistance += distance;

    if (prev.ele !== undefined && curr.ele !== undefined && curr.ele > prev.ele) {
      elevationGain += curr.ele - prev.ele;
    }

    if (prev.time && curr.time) {
      const timeDiff = (new Date(curr.time).getTime() - new Date(prev.time).getTime()) / 1000;
      const speed = calculateSpeed(distance, timeDiff);
      speeds.push(speed);
      maxSpeed = Math.max(maxSpeed, speed);
    }
  }

  const duration =
    points.length > 0 && points[0].time && points[points.length - 1].time
      ? (new Date(points[points.length - 1].time!).getTime() - new Date(points[0].time).getTime()) / 1000
      : 0;

  const avgSpeed = speeds.length > 0 ? speeds.reduce((sum, speed) => sum + speed, 0) / speeds.length : 0;

  return {
    name: trackName,
    points,
    distance: totalDistance,
    duration,
    elevationGain,
    maxSpeed,
    avgSpeed,
  };
}
