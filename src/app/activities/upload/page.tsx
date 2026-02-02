"use client";

import { useState, useRef, useCallback, useMemo, useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import {
  api,
  ActivityVisibility,
  ACTIVITY_TYPE_OPTIONS,
  ACTIVITY_TYPE_IDS,
  TrackPoint,
  StoppedSegment,
} from "@/lib/api";
import { DigTaggingModal } from "@/components/activity/dig-tagging-modal";
import {
  SegmentMergeModal,
  detectMergeableSegments,
  mergeAdjacentSegments,
} from "@/components/activity/segment-merge-modal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { TeamSelector } from "@/components/teams/team-selector";
import { parseGpxFile, getTimestampAtIndex, getTrackTimeRange } from "@/lib/gpx-parser";
import { ElevationProfile, MultiRangeSegment, getActivityTypeColor } from "@/components/activity/elevation-profile";

const VISIBILITY_OPTIONS: {
  value: ActivityVisibility;
  label: string;
  description: string;
  icon: React.ReactNode;
}[] = [
  {
    value: "public",
    label: "Public",
    description: "Anyone can view this activity",
    icon: (
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="w-5 h-5">
        <path
          fillRule="evenodd"
          d="M9.69 18.933l.003.001C9.89 19.02 10 19 10 19s.11.02.308-.066l.002-.001.006-.003.018-.008a5.741 5.741 0 00.281-.14c.186-.096.446-.24.757-.433.62-.384 1.445-.966 2.274-1.765C15.302 14.988 17 12.493 17 9A7 7 0 103 9c0 3.492 1.698 5.988 3.355 7.584a13.731 13.731 0 002.273 1.765 11.842 11.842 0 00.976.544l.062.029.018.008.006.003zM10 11.25a2.25 2.25 0 100-4.5 2.25 2.25 0 000 4.5z"
          clipRule="evenodd"
        />
      </svg>
    ),
  },
  {
    value: "private",
    label: "Private",
    description: "Only you can view this activity",
    icon: (
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="w-5 h-5">
        <path
          fillRule="evenodd"
          d="M10 1a4.5 4.5 0 00-4.5 4.5V9H5a2 2 0 00-2 2v6a2 2 0 002 2h10a2 2 0 002-2v-6a2 2 0 00-2-2h-.5V5.5A4.5 4.5 0 0010 1zm3 8V5.5a3 3 0 10-6 0V9h6z"
          clipRule="evenodd"
        />
      </svg>
    ),
  },
  {
    value: "teams_only",
    label: "Teams Only",
    description: "Share with selected teams",
    icon: (
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="w-5 h-5">
        <path d="M10 9a3 3 0 100-6 3 3 0 000 6zM6 8a2 2 0 11-4 0 2 2 0 014 0zM1.49 15.326a.78.78 0 01-.358-.442 3 3 0 014.308-3.516 6.484 6.484 0 00-1.905 3.959c-.023.222-.014.442.025.654a4.97 4.97 0 01-2.07-.655zM16.44 15.98a4.97 4.97 0 002.07-.654.78.78 0 00.357-.442 3 3 0 00-4.308-3.517 6.484 6.484 0 011.907 3.96 2.32 2.32 0 01-.026.654zM18 8a2 2 0 11-4 0 2 2 0 014 0zM5.304 16.19a.844.844 0 01-.277-.71 5 5 0 019.947 0 .843.843 0 01-.277.71A6.975 6.975 0 0110 18a6.974 6.974 0 01-4.696-1.81z" />
      </svg>
    ),
  },
];

export default function UploadActivityPage() {
  const router = useRouter();
  const { user, loading: authLoading } = useAuth();
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Basic form state
  const [name, setName] = useState("");
  const [activityType, setActivityType] = useState<string>(ACTIVITY_TYPE_IDS.RUN);
  const [visibility, setVisibility] = useState<ActivityVisibility>("public");
  const [selectedTeamIds, setSelectedTeamIds] = useState<string[]>([]);
  const [file, setFile] = useState<File | null>(null);
  const [error, setError] = useState("");
  const [uploading, setUploading] = useState(false);

  // Parsed GPX preview state
  const [parsedPoints, setParsedPoints] = useState<TrackPoint[]>([]);
  const [hasTimestamps, setHasTimestamps] = useState(false);
  const [parsing, setParsing] = useState(false);
  const [parseError, setParseError] = useState<string | null>(null);

  // Multi-sport mode state
  const [isMultiSport, setIsMultiSport] = useState(false);

  // Dig tagging modal state
  const [showDigModal, setShowDigModal] = useState(false);
  const [uploadedActivityId, setUploadedActivityId] = useState<string | null>(null);
  const [stoppedSegments, setStoppedSegments] = useState<StoppedSegment[]>([]);

  // Segment merge modal state
  const [showMergeModal, setShowMergeModal] = useState(false);
  const [pendingUpload, setPendingUpload] = useState(false);

  // Boundary indices: always includes 0 (start) and points.length-1 (end) implicitly
  // This array stores only the interior boundary indices
  const [boundaryIndices, setBoundaryIndices] = useState<number[]>([]);
  // Segment types: one activity type UUID per segment
  // Length = boundaryIndices.length + 1 (one segment between each pair of boundaries)
  const [segmentTypes, setSegmentTypes] = useState<string[]>([]);

  // Build segments from boundary indices and segment types for the elevation profile
  const multiRangeSegments = useMemo((): MultiRangeSegment[] => {
    if (!isMultiSport || parsedPoints.length === 0) return [];

    // All boundaries including start (0) and end (points.length - 1)
    const allBoundaries = [0, ...boundaryIndices, parsedPoints.length - 1];

    const segments: MultiRangeSegment[] = [];
    for (let i = 0; i < allBoundaries.length - 1; i++) {
      segments.push({
        startIndex: allBoundaries[i],
        endIndex: allBoundaries[i + 1],
        activityTypeId: segmentTypes[i] || activityType,
      });
    }

    return segments;
  }, [isMultiSport, parsedPoints.length, boundaryIndices, segmentTypes, activityType]);

  // Handle file selection and parsing
  const handleFileChange = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const selectedFile = e.target.files?.[0];
      if (!selectedFile) return;

      setFile(selectedFile);
      setParseError(null);
      setParsedPoints([]);
      setHasTimestamps(false);
      setIsMultiSport(false);
      setBoundaryIndices([]);
      setSegmentTypes([]);

      // Auto-fill name from filename
      if (!name) {
        setName(selectedFile.name.replace(/\.(gpx|fit|tcx)$/i, ""));
      }

      // Parse GPX for preview
      setParsing(true);
      try {
        const result = await parseGpxFile(selectedFile);
        setParsedPoints(result.points);
        setHasTimestamps(result.hasTimestamps);

        // Auto-fill name from GPX metadata if available
        if (result.name && !name) {
          setName(result.name);
        }
      } catch (err) {
        setParseError(err instanceof Error ? err.message : "Failed to parse GPX file");
      } finally {
        setParsing(false);
      }
    },
    [name],
  );

  // Handle clicking on the elevation profile to add a boundary
  const handleBoundaryClick = useCallback(
    (index: number) => {
      if (!isMultiSport) return;

      // Don't allow boundaries at the very start or end
      if (index <= 0 || index >= parsedPoints.length - 1) return;

      setBoundaryIndices((prev) => {
        // Check if there's already a boundary near this index (within 5 points)
        const nearbyIndex = prev.findIndex((b) => Math.abs(b - index) < 5);

        if (nearbyIndex !== -1) {
          // Remove existing nearby boundary
          const updated = [...prev];
          updated.splice(nearbyIndex, 1);

          // Also remove the corresponding segment type
          setSegmentTypes((prevTypes) => {
            const updatedTypes = [...prevTypes];
            // When removing boundary at index i, merge segments i and i+1
            // by removing the type at index i+1
            updatedTypes.splice(nearbyIndex + 1, 1);
            return updatedTypes;
          });

          return updated.sort((a, b) => a - b);
        } else {
          // Add new boundary
          const updated = [...prev, index].sort((a, b) => a - b);

          // Find position of new boundary to insert default type
          const newPos = updated.indexOf(index);
          setSegmentTypes((prevTypes) => {
            const updatedTypes = [...prevTypes];
            // Insert the default activity type after the new boundary
            updatedTypes.splice(newPos + 1, 0, activityType);
            return updatedTypes;
          });

          return updated;
        }
      });
    },
    [isMultiSport, parsedPoints.length, activityType],
  );

  // Toggle multi-sport mode
  const handleMultiSportToggle = useCallback(
    (enabled: boolean) => {
      setIsMultiSport(enabled);
      if (enabled) {
        // Initialize with a single segment using the primary activity type
        setBoundaryIndices([]);
        setSegmentTypes([activityType]);
      } else {
        setBoundaryIndices([]);
        setSegmentTypes([]);
      }
    },
    [activityType],
  );

  // Update segment type at a specific index
  const handleSegmentTypeChange = useCallback((segmentIndex: number, typeId: string) => {
    setSegmentTypes((prev) => {
      const updated = [...prev];
      updated[segmentIndex] = typeId;
      return updated;
    });
  }, []);

  // Remove a specific boundary
  const handleRemoveBoundary = useCallback((boundaryIndex: number) => {
    setBoundaryIndices((prev) => {
      const updated = [...prev];
      updated.splice(boundaryIndex, 1);
      return updated;
    });
    setSegmentTypes((prev) => {
      const updated = [...prev];
      // Merge segment types by removing the one after the removed boundary
      updated.splice(boundaryIndex + 1, 1);
      return updated;
    });
  }, []);

  // Handler for when dig tagging modal completes
  const handleDigModalClose = useCallback(() => {
    setShowDigModal(false);
    setUploadedActivityId(null);
    setStoppedSegments([]);
    router.push("/activities");
  }, [router]);

  // Check for adjacent same-sport segments and show merge modal if found
  const checkForMergeableSegments = useCallback((): boolean => {
    if (!isMultiSport || segmentTypes.length <= 1) {
      return false;
    }
    const mergeGroups = detectMergeableSegments(segmentTypes);
    return mergeGroups.length > 0;
  }, [isMultiSport, segmentTypes]);

  // Handle merge action from the modal
  const handleMerge = useCallback(() => {
    const result = mergeAdjacentSegments(boundaryIndices, segmentTypes);
    setBoundaryIndices(result.boundaryIndices);
    setSegmentTypes(result.segmentTypes);
    setShowMergeModal(false);
    setPendingUpload(true);
  }, [boundaryIndices, segmentTypes]);

  // Handle "keep as-is" from the modal - proceed with upload
  const handleKeepAsIs = useCallback(() => {
    setShowMergeModal(false);
    setPendingUpload(true);
  }, []);

  // Handle "edit" from the modal - close modal and stay on page
  const handleEditSegments = useCallback(() => {
    setShowMergeModal(false);
    setPendingUpload(false);
  }, []);

  // Actual upload function (called after merge decision)
  const performUpload = useCallback(async () => {
    if (!file || !user) return;

    setUploading(true);
    setError("");

    try {
      const teamIds = visibility === "teams_only" ? selectedTeamIds : undefined;

      // Build multi-sport data if enabled
      let typeBoundaries: string[] | undefined;
      let segmentTypeIds: string[] | undefined;

      if (isMultiSport && hasTimestamps && boundaryIndices.length > 0) {
        // Convert boundary indices to timestamps
        const { start, end } = getTrackTimeRange(parsedPoints);
        if (start && end) {
          // Build boundaries array: [start, ...interiorBoundaries, end]
          typeBoundaries = [
            start,
            ...boundaryIndices.map((idx) => getTimestampAtIndex(parsedPoints, idx) || ""),
            end,
          ].filter(Boolean);

          // Segment types (should have one fewer than boundaries)
          segmentTypeIds = segmentTypes;
        }
      }

      const activity = await api.uploadActivity(file, name, activityType, visibility, {
        teamIds,
        typeBoundaries,
        segmentTypes: segmentTypeIds,
      });

      // Check for stopped segments to offer dig tagging
      try {
        const segments = await api.getStoppedSegments(activity.id);
        if (segments.length > 0) {
          setUploadedActivityId(activity.id);
          setStoppedSegments(segments);
          setShowDigModal(true);
          return; // Don't navigate yet - wait for modal
        }
      } catch {
        // If fetching stopped segments fails, just continue to activities page
      }

      router.push("/activities");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Upload failed");
    } finally {
      setUploading(false);
      setPendingUpload(false);
    }
  }, [
    file,
    user,
    name,
    activityType,
    visibility,
    selectedTeamIds,
    isMultiSport,
    hasTimestamps,
    boundaryIndices,
    segmentTypes,
    parsedPoints,
    router,
  ]);

  // Trigger upload when pendingUpload becomes true
  // (after merge decision is made)
  useEffect(() => {
    if (pendingUpload) {
      performUpload();
    }
  }, [pendingUpload, performUpload]);

  // Auth check - must be after all hooks
  if (!authLoading && !user) {
    router.push("/login");
    return null;
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!file) {
      setError("Please select a file");
      return;
    }

    if (!name.trim()) {
      setError("Please enter a name");
      return;
    }

    if (visibility === "teams_only" && selectedTeamIds.length === 0) {
      setError("Please select at least one team to share with");
      return;
    }

    if (!user) {
      setError("Not authenticated");
      return;
    }

    // Check for mergeable segments before upload
    if (checkForMergeableSegments()) {
      setShowMergeModal(true);
      return;
    }

    // No merge needed, proceed with upload
    setPendingUpload(true);
  };

  return (
    <div className="max-w-2xl mx-auto">
      <Card>
        <CardHeader>
          <CardTitle>Upload Activity</CardTitle>
          <CardDescription>Upload a GPX file to add a new activity</CardDescription>
        </CardHeader>
        <form onSubmit={handleSubmit}>
          <CardContent className="space-y-6">
            {error && <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md">{error}</div>}

            <div className="space-y-2">
              <Label htmlFor="file">Activity File</Label>
              <div
                className="border-2 border-dashed rounded-lg p-8 text-center cursor-pointer hover:border-primary transition-colors"
                onClick={() => fileInputRef.current?.click()}
              >
                <input
                  ref={fileInputRef}
                  id="file"
                  type="file"
                  accept=".gpx,.fit,.tcx"
                  onChange={handleFileChange}
                  className="hidden"
                />
                {file ? (
                  <p className="text-sm font-medium">{file.name}</p>
                ) : (
                  <p className="text-muted-foreground">Click to select a GPX, FIT, or TCX file</p>
                )}
              </div>
              {parsing && <p className="text-sm text-muted-foreground">Parsing file...</p>}
              {parseError && <p className="text-sm text-destructive">{parseError}</p>}
            </div>

            <div className="space-y-2">
              <Label htmlFor="name">Activity Name</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Morning Run"
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="type">Activity Type</Label>
              <select
                id="type"
                value={activityType}
                onChange={(e) => setActivityType(e.target.value)}
                className="w-full h-10 px-3 py-2 border rounded-md bg-background text-foreground"
              >
                {ACTIVITY_TYPE_OPTIONS.map((type) => (
                  <option key={type.id} value={type.id}>
                    {type.name}
                  </option>
                ))}
              </select>
              <p className="text-xs text-muted-foreground">
                {isMultiSport ? "Primary activity type (used for display)" : "Type of activity"}
              </p>
            </div>

            {/* Elevation Profile Preview */}
            {parsedPoints.length > 0 && (
              <div className="space-y-2">
                <Label>Elevation Profile Preview</Label>
                <div className="border rounded-lg p-4">
                  <ElevationProfile
                    points={parsedPoints}
                    multiRangeMode={isMultiSport}
                    segments={multiRangeSegments}
                    onBoundaryClick={handleBoundaryClick}
                  />
                </div>
              </div>
            )}

            {/* Multi-sport toggle - only show if GPX has timestamps */}
            {parsedPoints.length > 0 && hasTimestamps && (
              <div className="space-y-4 p-4 border rounded-lg bg-muted/30">
                <div className="flex items-center gap-3">
                  <input
                    type="checkbox"
                    id="multi-sport"
                    checked={isMultiSport}
                    onChange={(e) => handleMultiSportToggle(e.target.checked)}
                    className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                  />
                  <div>
                    <Label htmlFor="multi-sport" className="cursor-pointer">
                      Multi-sport activity
                    </Label>
                    <p className="text-xs text-muted-foreground">
                      Define different activity types for different segments (e.g., bike + run)
                    </p>
                  </div>
                </div>

                {/* Segment type editors */}
                {isMultiSport && (
                  <div className="space-y-3 mt-4">
                    <Label>Segment Types</Label>
                    <div className="space-y-2">
                      {multiRangeSegments.map((segment, idx) => (
                        <div
                          key={idx}
                          className="flex items-center gap-3 p-2 rounded-md border"
                          style={{
                            borderLeftWidth: 4,
                            borderLeftColor: getActivityTypeColor(segment.activityTypeId),
                          }}
                        >
                          <span className="text-sm text-muted-foreground min-w-[80px]">Segment {idx + 1}</span>
                          <select
                            value={segment.activityTypeId}
                            onChange={(e) => handleSegmentTypeChange(idx, e.target.value)}
                            className="flex-1 h-8 px-2 text-sm border rounded-md bg-background text-foreground"
                          >
                            {ACTIVITY_TYPE_OPTIONS.map((type) => (
                              <option key={type.id} value={type.id}>
                                {type.name}
                              </option>
                            ))}
                          </select>
                          {/* Show remove button for interior boundaries (not first or last segment) */}
                          {idx > 0 && (
                            <Button
                              type="button"
                              variant="ghost"
                              size="sm"
                              className="h-8 px-2 text-muted-foreground hover:text-destructive"
                              onClick={() => handleRemoveBoundary(idx - 1)}
                              title="Remove boundary before this segment"
                            >
                              <svg
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 20 20"
                                fill="currentColor"
                                className="w-4 h-4"
                              >
                                <path
                                  fillRule="evenodd"
                                  d="M8.75 1A2.75 2.75 0 006 3.75v.443c-.795.077-1.584.176-2.365.298a.75.75 0 10.23 1.482l.149-.022.841 10.518A2.75 2.75 0 007.596 19h4.807a2.75 2.75 0 002.742-2.53l.841-10.519.149.023a.75.75 0 00.23-1.482A41.03 41.03 0 0014 4.193V3.75A2.75 2.75 0 0011.25 1h-2.5zM10 4c.84 0 1.673.025 2.5.075V3.75c0-.69-.56-1.25-1.25-1.25h-2.5c-.69 0-1.25.56-1.25 1.25v.325C8.327 4.025 9.16 4 10 4zM8.58 7.72a.75.75 0 00-1.5.06l.3 7.5a.75.75 0 101.5-.06l-.3-7.5zm4.34.06a.75.75 0 10-1.5-.06l-.3 7.5a.75.75 0 101.5.06l.3-7.5z"
                                  clipRule="evenodd"
                                />
                              </svg>
                            </Button>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Show notice when GPX doesn't have timestamps */}
            {parsedPoints.length > 0 && !hasTimestamps && (
              <div className="p-3 text-sm text-muted-foreground bg-muted/50 rounded-md">
                This GPX file does not contain timestamps. Multi-sport mode requires timestamp data.
              </div>
            )}

            <div className="space-y-3">
              <Label>Visibility</Label>
              <div className="grid gap-2">
                {VISIBILITY_OPTIONS.map((option) => (
                  <button
                    key={option.value}
                    type="button"
                    onClick={() => {
                      setVisibility(option.value);
                      if (option.value !== "teams_only") {
                        setSelectedTeamIds([]);
                      }
                    }}
                    className={`flex items-center gap-3 p-3 rounded-lg border-2 text-left transition-all ${
                      visibility === option.value
                        ? "border-primary bg-primary/5"
                        : "border-muted hover:border-muted-foreground/30"
                    }`}
                  >
                    <div className={`${visibility === option.value ? "text-primary" : "text-muted-foreground"}`}>
                      {option.icon}
                    </div>
                    <div className="flex-1">
                      <div className="font-medium">{option.label}</div>
                      <div className="text-sm text-muted-foreground">{option.description}</div>
                    </div>
                    <div
                      className={`w-4 h-4 rounded-full border-2 flex items-center justify-center ${
                        visibility === option.value ? "border-primary bg-primary" : "border-muted-foreground/30"
                      }`}
                    >
                      {visibility === option.value && <div className="w-2 h-2 rounded-full bg-primary-foreground" />}
                    </div>
                  </button>
                ))}
              </div>
            </div>

            {visibility === "teams_only" && (
              <div className="space-y-3 animate-in fade-in slide-in-from-top-2 duration-200">
                <Label>Share with Teams</Label>
                <TeamSelector
                  selectedTeamIds={selectedTeamIds}
                  onSelectionChange={setSelectedTeamIds}
                  disabled={uploading}
                />
                {selectedTeamIds.length > 0 && (
                  <p className="text-sm text-muted-foreground">
                    Sharing with {selectedTeamIds.length} team{selectedTeamIds.length > 1 ? "s" : ""}
                  </p>
                )}
              </div>
            )}

            <div className="flex gap-4">
              <Button type="button" variant="outline" className="flex-1" onClick={() => router.back()}>
                Cancel
              </Button>
              <Button type="submit" className="flex-1" disabled={uploading || parsing}>
                {uploading ? "Uploading..." : "Upload"}
              </Button>
            </div>
          </CardContent>
        </form>
      </Card>

      {/* Segment Merge Modal */}
      {showMergeModal && (
        <SegmentMergeModal
          segments={multiRangeSegments.map((seg, i) => ({
            index: i,
            activityTypeId: seg.activityTypeId,
            startIndex: seg.startIndex,
            endIndex: seg.endIndex,
          }))}
          onMerge={handleMerge}
          onKeepAsIs={handleKeepAsIs}
          onEdit={handleEditSegments}
        />
      )}

      {/* Dig Tagging Modal */}
      {showDigModal && uploadedActivityId && (
        <DigTaggingModal
          activityId={uploadedActivityId}
          stoppedSegments={stoppedSegments}
          onComplete={handleDigModalClose}
          onSkip={handleDigModalClose}
        />
      )}
    </div>
  );
}
