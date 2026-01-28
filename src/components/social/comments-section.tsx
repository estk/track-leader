"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { api, Comment } from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { formatDistanceToNow } from "@/lib/utils";

interface CommentsSectionProps {
  activityId: string;
  initialCommentCount: number;
}

export function CommentsSection({
  activityId,
  initialCommentCount,
}: CommentsSectionProps) {
  const { user } = useAuth();
  const [comments, setComments] = useState<Comment[]>([]);
  const [loading, setLoading] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [newComment, setNewComment] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [commentCount, setCommentCount] = useState(initialCommentCount);

  const loadComments = async () => {
    setLoading(true);
    try {
      const data = await api.getComments(activityId);
      setComments(data);
    } catch {
      // Error loading comments
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (expanded && comments.length === 0) {
      loadComments();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [expanded]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newComment.trim() || submitting) return;

    setSubmitting(true);
    try {
      const comment = await api.addComment(activityId, newComment.trim());
      setComments((prev) => [...prev, comment]);
      setCommentCount((c) => c + 1);
      setNewComment("");
    } catch {
      // Error adding comment
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (commentId: string) => {
    try {
      await api.deleteComment(commentId);
      setComments((prev) => prev.filter((c) => c.id !== commentId));
      setCommentCount((c) => Math.max(0, c - 1));
    } catch {
      // Error deleting
    }
  };

  return (
    <div className="space-y-3">
      <button
        onClick={() => setExpanded(!expanded)}
        className="text-sm text-muted-foreground hover:text-foreground"
      >
        {expanded ? "Hide" : "Show"} {commentCount} comment{commentCount !== 1 ? "s" : ""}
      </button>

      {expanded && (
        <div className="space-y-4">
          {loading ? (
            <p className="text-sm text-muted-foreground">Loading comments...</p>
          ) : (
            <>
              {comments.length === 0 && (
                <p className="text-sm text-muted-foreground">No comments yet</p>
              )}
              <div className="space-y-3">
                {comments.map((comment) => (
                  <div key={comment.id} className="flex gap-2">
                    <Link href={`/profile/${comment.user_id}`}>
                      <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center text-sm font-bold text-primary shrink-0">
                        {comment.user_name.charAt(0).toUpperCase()}
                      </div>
                    </Link>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <Link
                          href={`/profile/${comment.user_id}`}
                          className="font-medium text-sm hover:underline"
                        >
                          {comment.user_name}
                        </Link>
                        <span className="text-xs text-muted-foreground">
                          {formatDistanceToNow(new Date(comment.created_at))}
                        </span>
                        {user?.id === comment.user_id && (
                          <button
                            onClick={() => handleDelete(comment.id)}
                            className="text-xs text-muted-foreground hover:text-destructive ml-auto"
                          >
                            Delete
                          </button>
                        )}
                      </div>
                      <p className="text-sm mt-1">{comment.content}</p>
                    </div>
                  </div>
                ))}
              </div>

              {user && (
                <form onSubmit={handleSubmit} className="flex gap-2">
                  <Textarea
                    value={newComment}
                    onChange={(e) => setNewComment(e.target.value)}
                    placeholder="Add a comment..."
                    className="min-h-[60px] resize-none"
                  />
                  <Button type="submit" disabled={submitting || !newComment.trim()}>
                    {submitting ? "..." : "Post"}
                  </Button>
                </form>
              )}
            </>
          )}
        </div>
      )}
    </div>
  );
}
