# Uploading Activities

This guide covers everything you need to know about uploading activities to Track Leader.

## Supported File Formats

Track Leader currently supports:

- **GPX** (GPS Exchange Format) - The most common format exported by GPS devices and fitness apps

GPX files contain your GPS track including:
- Latitude/longitude coordinates
- Timestamps
- Elevation data (if available)

## Getting GPX Files

### From Garmin

1. Log into Garmin Connect
2. Go to Activities
3. Select your activity
4. Click the gear icon → Export to GPX

### From Strava

1. Log into Strava
2. Go to your activity
3. Click the three dots menu (⋯)
4. Select "Export GPX"

### From Apple Watch

Use a third-party app like:
- HealthFit
- RunGap

These apps can export your Apple Health workouts as GPX files.

### From Other Devices

Most GPS devices and apps support GPX export. Check your device's documentation for specific instructions.

## Uploading Process

### Step 1: Navigate to Upload

1. Log into Track Leader
2. Go to "Activities" in the main menu
3. Click "Upload Activity"

### Step 2: Select Your File

1. Click "Choose File" or drag and drop your GPX file
2. The file will begin uploading immediately

### Step 3: Add Details

While the file processes, you can add:

- **Title**: Name your activity (defaults to the date)
- **Description**: Add notes about your activity
- **Activity Type**: Select from Run, Ride, Hike, etc.
- **Privacy**: Choose public or private

### Step 4: Submit

Click "Upload" to submit your activity. Track Leader will:

1. Parse your GPX file
2. Extract the GPS track and timestamps
3. Calculate statistics (distance, elevation, pace)
4. Match against existing segments
5. Record any segment efforts

## Automatic Segment Matching

When you upload an activity, Track Leader automatically:

1. Compares your track against all segments
2. Finds segments where your route passed through the start and end points
3. Records an "effort" with your time for each matched segment
4. Updates leaderboards if you set a new record

### Matching Tolerance

Segment matching uses a tolerance of approximately 50 meters. Your track must pass within this distance of both the segment's start and end points to register an effort.

## Activity Types

Choose the correct activity type for accurate segment matching:

| Type | Icon | Description |
|------|------|-------------|
| Run | 🏃 | Running activities |
| Ride | 🚴 | Cycling activities |
| Hike | 🥾 | Hiking/walking |
| Trail Run | 🏔️ | Off-road running |
| MTB | 🚵 | Mountain biking |

Segments are often specific to an activity type. A running segment won't match against a cycling activity.

## Privacy Settings

Each activity can be set to:

- **Public**: Appears in the feed and on your profile
- **Private**: Only visible to you

Private activities still:
- Match against segments
- Appear on leaderboards
- Count toward your statistics

They just won't show up in other users' feeds.

## Troubleshooting

### "Invalid GPX file"

This usually means:
- The file is corrupted
- The file is not valid GPX format
- The file is empty

Try re-exporting from your source app.

### "No segments matched"

This can happen when:
- Your route didn't pass through any segments
- Your GPS track was too inaccurate
- The segment requires a different activity type

### "Activity already exists"

Track Leader detects duplicate uploads based on:
- Start time
- GPS coordinates

If you see this error, the activity is likely already in your account.

## Tips for Better Matching

1. **GPS Accuracy**: Use a device with good GPS accuracy
2. **Recording Interval**: 1-second recording captures more detail
3. **Start/Stop Points**: Begin recording before the segment start
4. **Activity Type**: Select the correct type for your activity

## Limits

- Maximum file size: 50 MB
- Maximum track points: 100,000
- Rate limit: 10 uploads per minute

For very long activities, consider splitting into multiple files.
