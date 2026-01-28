import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function NotFoundPage() {
  return (
    <div className="min-h-[60vh] flex items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 text-6xl">🔍</div>
          <CardTitle className="text-2xl">Page Not Found</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4 text-center">
          <p className="text-muted-foreground">
            The page you're looking for doesn't exist or has been moved.
          </p>
          <div className="flex gap-2 justify-center pt-4">
            <Button variant="outline" asChild>
              <Link href="/">Go Home</Link>
            </Button>
            <Button asChild>
              <Link href="/activities">View Activities</Link>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
