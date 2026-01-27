# Phase 4 Remaining Items

These items can be implemented later as polish/optimization:

## SSE Real-time Updates (Week 4)
- [ ] Create `GET /segments/{id}/leaderboard/stream` SSE endpoint
- [ ] Add broadcast mechanism for leaderboard changes when new efforts are recorded
- [ ] Create `useLeaderboardStream` hook in frontend
- [ ] Add rank change animations in leaderboard table

## Leaderboard Caching Service
- [ ] Create `crates/tracks/src/leaderboard_service.rs`
- [ ] Implement cache warming on segment creation
- [ ] Implement cache invalidation on new effort
- [ ] Add TTL management per scope (all_time vs week vs month)

## Notes
- SSE is nice-to-have; users can refresh manually for now
- Caching is a performance optimization; not needed until scale becomes an issue
- All core functionality is complete and ready for testing
