"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { api, Notification } from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { formatDistanceToNow } from "@/lib/utils";

function getNotificationIcon(type: string): string {
  switch (type) {
    case "follow":
      return "👤";
    case "kudos":
      return "👏";
    case "comment":
      return "💬";
    case "crown_achieved":
      return "👑";
    case "crown_lost":
      return "😢";
    case "pr":
      return "🏆";
    default:
      return "🔔";
  }
}

function getNotificationMessage(notification: Notification): string {
  switch (notification.notification_type) {
    case "follow":
      return `${notification.actor_name || "Someone"} started following you`;
    case "kudos":
      return `${notification.actor_name || "Someone"} gave kudos to your activity`;
    case "comment":
      return `${notification.actor_name || "Someone"} commented on your activity`;
    case "crown_achieved":
      return notification.message || "You achieved a new crown!";
    case "crown_lost":
      return notification.message || "You lost a crown";
    case "pr":
      return notification.message || "You set a new personal record!";
    default:
      return notification.message || "New notification";
  }
}

function getNotificationLink(notification: Notification): string | null {
  switch (notification.notification_type) {
    case "follow":
      return notification.actor_id ? `/profile/${notification.actor_id}` : null;
    case "kudos":
    case "comment":
      return notification.target_id ? `/activities/${notification.target_id}` : null;
    case "crown_achieved":
    case "crown_lost":
      return notification.target_id ? `/segments/${notification.target_id}` : null;
    case "pr":
      return notification.target_id ? `/activities/${notification.target_id}` : null;
    default:
      return null;
  }
}

export default function NotificationsPage() {
  const router = useRouter();
  const { user, loading: authLoading } = useAuth();
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [loading, setLoading] = useState(true);
  const [hasMore, setHasMore] = useState(true);
  const [unreadCount, setUnreadCount] = useState(0);

  useEffect(() => {
    if (!authLoading && !user) {
      router.push("/login");
    }
  }, [user, authLoading, router]);

  useEffect(() => {
    if (user) {
      loadNotifications();
    }
  }, [user]);

  const loadNotifications = async () => {
    setLoading(true);
    try {
      const response = await api.getNotifications(50, 0);
      setNotifications(response.notifications);
      setUnreadCount(response.unread_count);
      setHasMore(response.notifications.length === 50);
    } catch {
      // Error loading notifications
    } finally {
      setLoading(false);
    }
  };

  const loadMore = async () => {
    try {
      const response = await api.getNotifications(50, notifications.length);
      setNotifications((prev) => [...prev, ...response.notifications]);
      setHasMore(response.notifications.length === 50);
    } catch {
      // Error loading more
    }
  };

  const handleMarkRead = async (notificationId: string) => {
    try {
      await api.markNotificationRead(notificationId);
      setNotifications((prev) =>
        prev.map((n) =>
          n.id === notificationId ? { ...n, read_at: new Date().toISOString() } : n
        )
      );
      setUnreadCount((c) => Math.max(0, c - 1));
    } catch {
      // Error marking as read
    }
  };

  const handleMarkAllRead = async () => {
    try {
      await api.markAllNotificationsRead();
      setNotifications((prev) =>
        prev.map((n) => ({ ...n, read_at: n.read_at || new Date().toISOString() }))
      );
      setUnreadCount(0);
    } catch {
      // Error marking all as read
    }
  };

  if (authLoading || !user) {
    return (
      <div className="container mx-auto px-4 py-8">
        <p className="text-muted-foreground">Loading...</p>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8 max-w-2xl">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Notifications</h1>
        {unreadCount > 0 && (
          <Button variant="outline" size="sm" onClick={handleMarkAllRead}>
            Mark all as read ({unreadCount})
          </Button>
        )}
      </div>

      {loading ? (
        <div className="text-center py-8 text-muted-foreground">Loading...</div>
      ) : notifications.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-muted-foreground">No notifications yet</p>
            <p className="text-sm text-muted-foreground mt-2">
              Follow other users to see their activities in your feed
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2">
          {notifications.map((notification) => {
            const link = getNotificationLink(notification);
            const isUnread = !notification.read_at;
            const icon = getNotificationIcon(notification.notification_type);
            const message = getNotificationMessage(notification);

            const content = (
              <Card
                className={`hover:bg-muted/50 transition-colors ${
                  isUnread ? "border-primary/30 bg-primary/5" : ""
                }`}
              >
                <CardContent className="p-4">
                  <div className="flex items-start gap-3">
                    <span className="text-2xl">{icon}</span>
                    <div className="flex-1 min-w-0">
                      <p className={`${isUnread ? "font-medium" : ""}`}>{message}</p>
                      <p className="text-sm text-muted-foreground mt-1">
                        {formatDistanceToNow(new Date(notification.created_at))}
                      </p>
                    </div>
                    {isUnread && (
                      <span className="w-2 h-2 bg-primary rounded-full shrink-0 mt-2" />
                    )}
                  </div>
                </CardContent>
              </Card>
            );

            return link ? (
              <Link
                key={notification.id}
                href={link}
                onClick={() => {
                  if (isUnread) handleMarkRead(notification.id);
                }}
              >
                {content}
              </Link>
            ) : (
              <button
                key={notification.id}
                onClick={() => {
                  if (isUnread) handleMarkRead(notification.id);
                }}
                className="w-full text-left"
              >
                {content}
              </button>
            );
          })}

          {hasMore && (
            <div className="text-center pt-4">
              <Button variant="outline" onClick={loadMore}>
                Load more
              </Button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
