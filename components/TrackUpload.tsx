"use client";

import { useState } from "react";
import { Track } from "@/lib/database";
import { CloudArrowUpIcon } from "@heroicons/react/24/outline";

interface TrackUploadProps {
  onTrackUploaded: (track: Track) => void;
}

export default function TrackUpload({ onTrackUploaded }: TrackUploadProps) {
  const [uploading, setUploading] = useState(false);
  const [dragOver, setDragOver] = useState(false);

  const handleFileUpload = async (file: File) => {
    if (!file.name.toLowerCase().endsWith(".gpx")) {
      alert("Please select a GPX file");
      return;
    }

    setUploading(true);

    try {
      const formData = new FormData();
      formData.append("file", file);

      const response = await fetch("/api/tracks", {
        method: "POST",
        body: formData,
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Upload failed");
      }

      const track = await response.json();
      onTrackUploaded(track);
      alert("Track uploaded successfully!");
    } catch (error) {
      console.error("Upload error:", error);
      alert(`Upload failed: ${error instanceof Error ? error.message : "Unknown error"}`);
    } finally {
      setUploading(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);

    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) {
      handleFileUpload(files[0]);
    }
  };

  const handleFileInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files && files.length > 0) {
      handleFileUpload(files[0]);
    }
    e.target.value = "";
  };

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <h2 className="text-xl font-semibold mb-4">Upload Track</h2>

      <div
        className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
          dragOver ? "border-blue-500 bg-blue-50" : "border-gray-300 hover:border-gray-400"
        }`}
        onDragOver={(e) => {
          e.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={handleDrop}
      >
        {uploading ? (
          <div className="space-y-2">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"></div>
            <p className="text-gray-600">Uploading and processing track...</p>
          </div>
        ) : (
          <div className="space-y-4">
            <CloudArrowUpIcon width={25} className="h-1 w-1 text-gray-400" />
            <div>
              <p className="text-gray-600">
                Drop your GPX file here, or{" "}
                <label className="text-blue-500 hover:text-blue-700 cursor-pointer">
                  browse
                  <input type="file" className="hidden" accept=".gpx" onChange={handleFileInput} />
                </label>
              </p>
              <p className="text-sm text-gray-500">GPX files only</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
