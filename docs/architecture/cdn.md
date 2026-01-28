# CDN Configuration Guide

Track Leader uses Cloudflare as a CDN for optimal performance and security.

## Recommended Cloudflare Settings

### Caching

**Page Rules:**

1. **Static Assets** (`/_next/static/*`)
   - Cache Level: Cache Everything
   - Edge TTL: 1 month
   - Browser TTL: 1 year

2. **Images** (`/images/*`)
   - Cache Level: Cache Everything
   - Edge TTL: 1 week
   - Browser TTL: 1 week

3. **API Routes** (`/api/*`)
   - Cache Level: Bypass Cache
   - (Let the backend handle caching headers)

### Performance

- **Auto Minify:** Enable for JavaScript, CSS, HTML
- **Brotli Compression:** Enable
- **Early Hints:** Enable
- **Rocket Loader:** Disable (may interfere with Next.js)
- **HTTP/3 (QUIC):** Enable

### Security

- **SSL/TLS:** Full (Strict)
- **Always Use HTTPS:** Enable
- **Minimum TLS Version:** 1.2
- **HSTS:** Enable with max-age of 1 year

### Speed Optimization

- **Polish:** Lossless (for images)
- **Mirage:** Enable (lazy-loads images)
- **Argo Smart Routing:** Enable if available

## Next.js Cache Headers

Cache headers are configured in `next.config.js`:

```javascript
async headers() {
  return [
    {
      source: '/_next/static/:path*',
      headers: [
        {
          key: 'Cache-Control',
          value: 'public, max-age=31536000, immutable',
        },
      ],
    },
    {
      source: '/images/:path*',
      headers: [
        {
          key: 'Cache-Control',
          value: 'public, max-age=604800, stale-while-revalidate=86400',
        },
      ],
    },
  ];
}
```

## Backend Cache Headers

The Rust backend includes gzip compression via `tower-http`. Additional cache headers should be added to specific endpoints:

- **GET /segments/:id/track**: Cache for 1 hour (track data rarely changes)
- **GET /segments/:id/leaderboard**: Cache for 5 minutes (changes with new efforts)
- **GET /activities/:id/track**: Cache for 24 hours (immutable after upload)

## DNS Configuration

1. Create A/AAAA records pointing to your server
2. Enable Cloudflare proxy (orange cloud)
3. Ensure SSL certificates are properly configured

## Monitoring

Use Cloudflare Analytics to monitor:
- Cache hit ratio (target: >80%)
- Response times
- Geographic distribution of requests
- Security threats blocked

## Troubleshooting

**Low cache hit ratio:**
- Check that Page Rules are correctly configured
- Ensure dynamic content isn't being cached
- Verify query strings aren't breaking cache keys

**SSL errors:**
- Verify origin server has valid certificate
- Check SSL/TLS encryption mode matches server config

**Performance issues:**
- Enable Argo if available
- Check for render-blocking resources
- Verify compression is working (check response headers)
