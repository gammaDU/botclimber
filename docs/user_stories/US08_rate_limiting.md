### US08: Fair Play and Rate Limiting
**As a** Core Developer,
**I want** to implement robust rate limiting and flood protection on the API,
**So that** the server remains stable even if a user's bot accidentally enters an infinite loop or floods the system with requests.

**Acceptance Criteria:**
- **Global Rate Limiting**: The system implements a global rate limit per IP address to prevent basic Denial-of-Service (DoS) scenarios, even before authentication.
- **Bot Rate Limiting**: The system enforces a specific rate limit per `X-API-Key` for gameplay commands (Join, Jump, State).
- **Graceful Handling of Floods**: If a bot exceeds its quota, the server MUST return a `429 Too Many Requests` response instead of processing the request, saving CPU and DB resources.
- **Burst Allowance**: The rate limiter supports a "token bucket" approach or similar, allowing for short bursts of requests (e.g., 5 requests in a single tick) while maintaining a lower average frequency.
- **Header Feedback**: Responses include standard `RateLimit-*` headers to inform bot developers of their remaining quota and reset time.
- **Configurability**: Administrators can adjust these limits via the `Admin API` or server configuration without restarting the service.
- **Internal Protection**: Internal components (like the Game Engine) are protected from malformed or high-frequency data from the API layer.
