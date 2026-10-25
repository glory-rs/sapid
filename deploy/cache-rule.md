# Cloudflare cache rule

Zone: `taidge.org` (same account as both Pages projects).

- Name: **Glory and Sapid origin browser caching**
- Rule ID: `6c2629ddaf9947489b5f6fe64c54121e`
- Status: Active
- Cache eligibility: Eligible for cache
- Edge TTL: Use cache-control header if present, bypass cache if not
- Browser TTL: Respect origin TTL
- Other cache settings: Unchanged

Expression:

```text
(http.host in {"glory.taidge.org" "sapid.taidge.org"}) and
(starts_with(http.request.uri.path, "/assets-") or
 starts_with(http.request.uri.path, "/media-") or
 ends_with(http.request.uri.path, ".css") or
 ends_with(http.request.uri.path, ".js") or
 ends_with(http.request.uri.path, ".wasm") or
 ends_with(http.request.uri.path, ".svg") or
 ends_with(http.request.uri.path, ".ico") or
 ends_with(http.request.uri.path, ".png") or
 ends_with(http.request.uri.path, ".json"))
```

Entry HTML remains outside this rule and keeps Pages' default revalidation.
The rule fixes the zone's default four-hour browser TTL overriding shorter
origin directives on CSS, JS and deployment manifests. Immutable directories
retain their one-year TTL. It does not change other hostnames in this zone.

Manage it in **taidge.org → Caching → Cache Rules**. This reference is shared by
the website and Sapid repositories; Pages deployments do not edit zone settings.
