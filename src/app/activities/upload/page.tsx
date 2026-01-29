"use client";

import { useState, useRef } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { api, ActivityVisibility, ACTIVITY_TYPE_OPTIONS, ACTIVITY_TYPE_IDS } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { TeamSelector } from "@/components/teams/team-selector";


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
        <path fillRule="evenodd" d="M9.69 18.933l.003.001C9.89 19.02 10 19 10 19s.11.02.308-.066l.002-.001.006-.003.018-.008a5.741 5.741 0 00.281-.14c.186-.096.446-.24.757-.433.62-.384 1.445-.966 2.274-1.765C15.302 14.988 17 12.493 17 9A7 7 0 103 9c0 3.492 1.698 5.988 3.355 7.584a13.731 13.731 0 002.273 1.765 11.842 11.842 0 00.976.544l.062.029.018.008.006.003zM10 11.25a2.25 2.25 0 100-4.5 2.25 2.25 0 000 4.5z" clipRule="evenodd" />
      </svg>
    ),
  },
  {
    value: "private",
    label: "Private",
    description: "Only you can view this activity",
    icon: (
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="w-5 h-5">
        <path fillRule="evenodd" d="M10 1a4.5 4.5 0 00-4.5 4.5V9H5a2 2 0 00-2 2v6a2 2 0 002 2h10a2 2 0 002-2v-6a2 2 0 00-2-2h-.5V5.5A4.5 4.5 0 0010 1zm3 8V5.5a3 3 0 10-6 0V9h6z" clipRule="evenodd" />
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

  const [name, setName] = useState("");
  const [activityType, setActivityType] = useState<string>(ACTIVITY_TYPE_IDS.RUN);
  const [visibility, setVisibility] = useState<ActivityVisibility>("public");
  const [selectedTeamIds, setSelectedTeamIds] = useState<string[]>([]);
  const [file, setFile] = useState<File | null>(null);
  const [error, setError] = useState("");
  const [uploading, setUploading] = useState(false);

  if (!authLoading && !user) {
    router.push("/login");
    return null;
  }

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0];
    if (selectedFile) {
      setFile(selectedFile);
      if (!name) {
        setName(selectedFile.name.replace(/\.(gpx|fit|tcx)$/i, ""));
      }
    }
  };

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

    setUploading(true);

    if (!user) {
      setError("Not authenticated");
      return;
    }

    try {
      const teamIds = visibility === "teams_only" ? selectedTeamIds : undefined;
      await api.uploadActivity(file, name, activityType, visibility, { teamIds });
      router.push("/activities");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Upload failed");
    } finally {
      setUploading(false);
    }
  };

  return (
    <div className="max-w-xl mx-auto">
      <Card>
        <CardHeader>
          <CardTitle>Upload Activity</CardTitle>
          <CardDescription>
            Upload a GPX file to add a new activity
          </CardDescription>
        </CardHeader>
        <form onSubmit={handleSubmit}>
          <CardContent className="space-y-6">
            {error && (
              <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md">
                {error}
              </div>
            )}

            <div className="space-y-2">
              <Label htmlFor="file">GPX File</Label>
              <div
                className="border-2 border-dashed rounded-lg p-8 text-center cursor-pointer hover:border-primary transition-colors"
                onClick={() => fileInputRef.current?.click()}
              >
                <input
                  ref={fileInputRef}
                  id="file"
                  type="file"
                  accept=".gpx"
                  onChange={handleFileChange}
                  className="hidden"
                />
                {file ? (
                  <p className="text-sm font-medium">{file.name}</p>
                ) : (
                  <p className="text-muted-foreground">
                    Click to select a GPX file
                  </p>
                )}
              </div>
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
            </div>

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
                      <div className="text-sm text-muted-foreground">
                        {option.description}
                      </div>
                    </div>
                    <div
                      className={`w-4 h-4 rounded-full border-2 flex items-center justify-center ${
                        visibility === option.value
                          ? "border-primary bg-primary"
                          : "border-muted-foreground/30"
                      }`}
                    >
                      {visibility === option.value && (
                        <div className="w-2 h-2 rounded-full bg-primary-foreground" />
                      )}
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
              <Button
                type="button"
                variant="outline"
                className="flex-1"
                onClick={() => router.back()}
              >
                Cancel
              </Button>
              <Button type="submit" className="flex-1" disabled={uploading}>
                {uploading ? "Uploading..." : "Upload"}
              </Button>
            </div>
          </CardContent>
        </form>
      </Card>
    </div>
  );
}
