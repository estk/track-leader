"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuth } from "@/lib/auth-context";
import { api, UserWithDemographics, UpdateDemographicsRequest } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";

type Gender = "male" | "female" | "other" | "prefer_not_to_say" | "";

export default function SettingsPage() {
  const router = useRouter();
  const { user, loading: authLoading } = useAuth();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  const [gender, setGender] = useState<Gender>("");
  const [birthYear, setBirthYear] = useState<string>("");
  const [weightKg, setWeightKg] = useState<string>("");
  const [country, setCountry] = useState<string>("");
  const [region, setRegion] = useState<string>("");

  useEffect(() => {
    if (!authLoading && !user) {
      router.push("/login");
      return;
    }

    if (user) {
      api.getMyDemographics()
        .then((data: UserWithDemographics) => {
          setGender((data.gender as Gender) || "");
          setBirthYear(data.birth_year?.toString() || "");
          setWeightKg(data.weight_kg?.toString() || "");
          setCountry(data.country || "");
          setRegion(data.region || "");
        })
        .catch(() => {
          setMessage({ type: "error", text: "Failed to load demographics" });
        })
        .finally(() => setLoading(false));
    }
  }, [user, authLoading, router]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setMessage(null);
    setSaving(true);

    const data: UpdateDemographicsRequest = {
      gender: gender || null,
      birth_year: birthYear ? parseInt(birthYear, 10) : null,
      weight_kg: weightKg ? parseFloat(weightKg) : null,
      country: country || null,
      region: region || null,
    };

    try {
      await api.updateMyDemographics(data);
      setMessage({ type: "success", text: "Demographics saved successfully" });
    } catch (err) {
      setMessage({ type: "error", text: err instanceof Error ? err.message : "Failed to save demographics" });
    } finally {
      setSaving(false);
    }
  };

  if (authLoading || loading) {
    return (
      <div className="max-w-2xl mx-auto space-y-6">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-96 w-full" />
      </div>
    );
  }

  if (!user) {
    return null;
  }

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div className="flex items-center gap-4">
        <Link
          href="/profile"
          className="text-muted-foreground hover:text-foreground transition-colors"
        >
          &larr; Back to Profile
        </Link>
      </div>

      <h1 className="text-3xl font-bold">Settings</h1>

      <Card>
        <CardHeader>
          <CardTitle>Demographics</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            <div className="space-y-2">
              <Label htmlFor="gender">Gender</Label>
              <select
                id="gender"
                value={gender}
                onChange={(e) => setGender(e.target.value as Gender)}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              >
                <option value="">Select gender</option>
                <option value="male">Male</option>
                <option value="female">Female</option>
                <option value="other">Other</option>
                <option value="prefer_not_to_say">Prefer not to say</option>
              </select>
              <p className="text-sm text-muted-foreground">
                Used for gender-specific leaderboards (KOM/QOM)
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="birthYear">Birth Year</Label>
              <Input
                id="birthYear"
                type="number"
                min="1900"
                max={new Date().getFullYear()}
                placeholder="e.g., 1990"
                value={birthYear}
                onChange={(e) => setBirthYear(e.target.value)}
              />
              <p className="text-sm text-muted-foreground">
                Used for age group leaderboards
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="weightKg">Weight (kg)</Label>
              <Input
                id="weightKg"
                type="number"
                min="0"
                step="0.1"
                placeholder="e.g., 70.5"
                value={weightKg}
                onChange={(e) => setWeightKg(e.target.value)}
              />
              <p className="text-sm text-muted-foreground">
                Optional, used for power calculations
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="country">Country</Label>
              <Input
                id="country"
                type="text"
                placeholder="e.g., United States"
                value={country}
                onChange={(e) => setCountry(e.target.value)}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="region">Region / State</Label>
              <Input
                id="region"
                type="text"
                placeholder="e.g., California"
                value={region}
                onChange={(e) => setRegion(e.target.value)}
              />
            </div>

            {message && (
              <div
                className={`p-3 rounded-md text-sm ${
                  message.type === "success"
                    ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                    : "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
                }`}
              >
                {message.text}
              </div>
            )}

            <Button type="submit" disabled={saving} className="w-full">
              {saving ? "Saving..." : "Save Demographics"}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
